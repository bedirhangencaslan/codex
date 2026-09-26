# DESTEK — Kurumsal Destek Talep (Ticket) Sistemi

Her şirketin kurabileceği tam teşekküllü yardım masası çekirdeği: otomatik
önceliklendirme, ekip yönlendirme ve temsilci atama, iş-saati temelli SLA,
olay yeniden-oynatma ile yaşam döngüsü, eskalasyon, denetim izi ve metrik raporu.

Çözüm: `solution.py` (yalnız standart kütüphane). Tüm zaman damgaları ISO
`YYYY-MM-DDTHH:MM` (yerel, tz'siz). Tüm sıralamalar deterministiktir.

## Veri (fixtures/)

- `agents.json`: `{id, name, team, skills: [kategori...], max_open, senior: bool}`
- `teams.json`: `{kategori: takım}` yönlendirme tablosu
- `tickets.json`: `{id, subject, description, customer, plan ("kurumsal"|"standart"),
  channel ("eposta"|"portal"|"telefon"), category, priority (null olabilir), created_at, tags: []}`
- `sla.json`: `{"P1": {"first_response": dk, "resolve": dk}, ...}` (İŞ dakikası)
- `holidays.json`: `["YYYY-MM-DD", ...]`
- `events.json`: zaman sıralı olay listesi (aşağıda)

## 1. Önceliklendirme (`derive_priority(ticket) -> "P1".."P4"`)

`priority` alanı null değilse aynen kullanılır. Null ise:

1. Taban: `subject + " " + description` içinde (`str.lower()` ile) şu kelimelerden
   biri geçiyorsa **P1**: `çöktü, kesinti, veri kaybı, güvenlik`.
   Yoksa şunlardan biri geçiyorsa **P2**: `hata, çalışmıyor, yavaş`.
   Hiçbiri yoksa **P3**.
2. `channel == "telefon"` ise bir seviye yükselt (P3→P2 gibi; P1 tavandır).
3. `plan == "kurumsal"` ise bir seviye yükselt.

## 2. Yönlendirme ve atama (`route(ticket, agents, teams, open_counts) -> agent_id | None`)

- Takım: `teams[ticket.category]`.
- Adaylar: o takımda `skills` listesinde `ticket.category` bulunan ve
  `open_counts[agent] < max_open` olan temsilciler; sıralama
  `(open_counts[agent] artan, id artan)`; ilki seçilir.
- Beceri eşleşmesi yoksa: aynı takımda kapasitesi olan herhangi biri (aynı sıralama).
- Kimse yoksa `None` (atanmamış).
- Toplu atamada biletler `(created_at, id)` artan sırayla işlenir ve `open_counts`
  her atamayla güncellenir (`assign_all(tickets, agents, teams) -> {ticket_id: agent_id|None}`).

## 3. İş saati ve SLA

Mesai: hafta içi (Pzt–Cum) **09:00–18:00**; `holidays.json` günleri mesai dışı.

```python
business_minutes_between(a: str, b: str, holidays) -> int   # a<=b, mesai kesişimi
add_business_minutes(start: str, minutes: int, holidays) -> str  # ISO dakika (":SS" yok)
```

- `add_business_minutes`: mesai dışı başlangıç bir sonraki mesai açılışına yuvarlanır;
  sonuç tam mesai bitimine (18:00) denk gelirse 18:00 yazılır (ertesi güne taşmaz).
- SLA süreleri **öncelik türetildikten sonra** `sla.json`'dan alınır; son teslim:
  `deadline = add_business_minutes(created_at, süre)`.

## 4. Yaşam döngüsü — olay yeniden-oynatma (`replay(tickets, events, agents, teams, sla, holidays) -> state`)

Durumlar: `new → open → pending → resolved → closed` + `resolved → open` (yeniden açma).

Olaylar (`events.json`, `ts` artan): `{ts, ticket_id, type, ...}`

| type | etki |
|---|---|
| `agent_reply` | ilk yanıtsa `first_response_at = ts`; durum `new` ise `open` olur |
| `customer_reply` | durum `pending` ise `open`; durum `resolved` ise ve `ts - resolved_at ≤ 14 gün` ise **yeniden açılır** (`open`, `reopen_count += 1`, `resolved_at` null'a döner) |
| `status` (`to` alanı) | izinli geçişler: open→pending, pending→open, open→resolved, pending→resolved, resolved→closed. İzinsiz geçiş **uygulanmaz**, denetime `{"ts","ticket_id","error":"invalid_transition"}` yazılır |
| `tag` (`value`) | `tags`'e eklenir (tekrarsız) |

**Eskalasyon**: her olay uygulanmadan ÖNCE, tüm biletler için bakılır: durumu
`resolved/closed` olmayan, henüz eskale edilmemiş ve `resolve_deadline < ts`
olan her bilet eskale edilir → öncelik bir seviye yükselir (P1 tavan),
`escalated = true`, denetime `{"ts","ticket_id","action":"escalated"}` yazılır.
(Eskalasyon SLA sürelerini ve son teslimleri DEĞİŞTİRMEZ.)

Başlangıç durumu: her bilet `new`, atama Bölüm 2'deki `assign_all` ile yapılır.

`state`: `{ticket_id: {"status", "assignee", "priority", "first_response_at" (yok→null),
"resolved_at" (yok→null), "reopen_count", "escalated", "tags": sıralı liste,
"first_response_deadline", "resolve_deadline"}}` + `state["_audit"]`: denetim listesi (ts sırası).

## 5. Rapor CLI

```
python3 solution.py report --fixtures <DIR> --now YYYY-MM-DDTHH:MM
```

Yeniden-oynatma sonrası (`now` yalnız açık-SLA kontrolünde kullanılmaz; rapor
salt replay sonucudur) `json.dumps` ile:

```json
{
  "open_by_team": {"takım": açık_sayısı},      // status open|pending|new; yalnız >0
  "first_response": {"met": n, "breached": n}, // yanıtlananlarda: ts <= deadline ?
  "escalated": ["id", ...],                     // id artan
  "reopened": ["id", ...],                      // reopen_count>0, id artan
  "unassigned": ["id", ...],                    // assignee null, id artan
  "agent_open": {"agent_id": n}                 // open|pending|new biletler; yalnız >0
}
```
