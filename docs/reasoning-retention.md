# Düşünme kırpma (reasoning retention)

Kod tabanındaki adı `reasoning_retention`, özellik bayrağı `reasoning_cost_model`. Aramızda
"reasoning shrinker" / "düşünme kırpma" diye geçiyor, ama isim biraz yanıltıcı: metni kısaltmıyor,
özetlemiyor. Her düşünme bloğu için tek bir karar veriyor: **ya bağlamda aynen kalır ya tamamen
çıkar.**

Bir cümleyle: *bir kullanıcı turu bittiğinde, o turun düşünmelerini bağlamda taşımanın mı yoksa
atmanın mı daha ucuz olduğunu hesaplar, kararı dondurur ve bir daha değiştirmez.*

---

## Hangi sorunu çözüyor

Model her istekte konuşmanın tamamını yeniden alır. Sağlayıcı, bir önceki istekle **baştan
itibaren birebir aynı** olan kısmı önbellekten okur ve ucuza faturalar. İlk farklı token'dan
sonrasını ise tam fiyattan yeniden okur (re-prefill).

Eski davranış basitti: bir tur bitince o turun düşünmesini at, bağlam küçülsün. Bunun gözden
kaçırdığı şey, Chat protokolünde düşünmenin ayrı bir kalem olmaması. Düşünme, açıkladığı asistan
mesajının veya araç çağrısının **içine** gömülü gider. Onu atmak o mesajı yeniden yazmak demek;
önbellek o noktadan sonuna kadar geçersiz olur:

```
Önceki istek:  [sistem][tur 1: kullanıcı | düşünme+araç | sonuç | ...][tur 2 ...]
Bu istek:      [sistem][tur 1: kullanıcı | ------araç    | sonuç | ...][tur 2 ...]
                                           ^
                         ilk fark: buradan sona kadar her şey tam fiyattan yeniden okunur
```

Bir turun ilk düşünmesi o turun en başında durduğu için, onu atmak turun **tamamını** yeniden
faturalar. Gerçek bir oturumda `low` efor düşünmelerinin medyanı 32 token çıktı: 32 token'ı
önbellekte taşımak yuvarlama hatası kadar tutuyor, atmak ise beş haneli bir re-prefill. `max`
eforda düşünme yüzlerce kat büyük ve hesap tersine dönüyor. Yani "atmak her zaman ucuzdur" doğru
değil; bu bir sabit değil, hesaplanması gereken bir şey.

## Nerede ve ne zaman çalışır

| Adım | Nerede | Ne yapar |
|---|---|---|
| Etiketleme | `core/src/session/mod.rs` | Kaydedilen her düşünme öğesine sahibi olan turun kimliğini yazar (`reasoning_turn`). |
| Tetikleme | `core/src/session/turn.rs` | Her örnekleme isteğinden önce, prompt kurulmadan hemen önce `sess.freeze_reasoning_retention(...)` çağrılır. |
| Karar | `core/src/session/mod.rs` → `context_manager/history.rs` → `core/src/reasoning_retention.rs` | Bitmiş turlara ait, henüz kararı verilmemiş düşünmeleri fiyatlar ve `reasoning_retained = true/false` damgasını basar. |
| Uygulama | `context_manager/history.rs` → `for_prompt_annotated` / `dropped_reason` | Prompt kurulurken `false` damgalı düşünmeler çıkarılır, `true` damgalılar kalır. |

Çağrı her istekte yapılır, ama yalnızca **kararı verilmemiş** düşünmelere dokunur. Pratikte her
turun düşünmesi bir kez fiyatlanır: bir sonraki kullanıcı turunun ilk isteğinden hemen önce. O an
düşünmenin boyu kesinleşmiştir ve henüz hiçbir istek onun bir versiyonuna bağlanmamıştır.

Fiyatlama yapılamadığında eski davranışa düşülür (`drop_completed_reasoning`): biten turun
düşünmesi doğrudan atılır. Bu iki durumda olur:

- `reasoning_cost_model` bayrağı kapalıysa (varsayılan: **açık**),
- kalan bağlam bütçesi bilinmiyorsa (`base_window_tokens_remaining` yoksa).

## Karar nasıl veriliyor

### Ufuk (horizon): taşınan bir token kaç kez daha ödenecek?

Tutulan bir düşünme token'ı, bir sonraki compaction'a kadar her istekte önbellekten tekrar okunur.
Compaction öneki zaten baştan yazacağı için ondan sonrası sayılmaz.

```
ufuk = round( kalan_bütçe / WINDOW_TOKENS × pencere_başına_istek )   ∈ [0, 200]
```

- `WINDOW_TOKENS = 80_000` (`request_density.rs`)
- `pencere_başına_istek`: kullanıcının ölçülen istek yoğunluğu (`RequestDensity`); ölçüm yokken
  `70`.
- Ufuk `0` ise (bütçe bitmiş) tutmanın bir bedeli yok, ama atmanın getirisi de yok: compaction
  öneki birazdan zaten yazacak.

### Kırılma maliyeti

Karar "hangi düşünmeyi at" değil, **"öneki nereden kıralım"** sorusu. Bir noktadan kırıldığında
oradan sonraki bütün kararsız düşünmeler de bedavaya atılabilir (cascade). Her aday kırılma
noktası için:

```
maliyet = sonraki_kısım × (1 − 0,2)  +  ufuk × 0,2 × tutulan_düşünme
          └ bir kez tam fiyattan       └ tutulanlar ufuk boyunca önbellekten
            yeniden okunacak kısım       okunmaya devam eder
```

`0,2` = önbellekli girdinin tam fiyata oranı (`CACHED_INPUT_PRICE_RATIO`). Z.ai'de
`glm-5.3-flash` için önbellekli girdi, taze girdinin beşte biri.

Algoritma en yeni adaydan geriye doğru yürür. Başlangıç noktası "hiçbirini atma"dır; her adımda bir
düşünmeyi daha atmayı dener ve en düşük maliyetli kırılma noktasını seçer. Beraberlikte atmayı
seçer. Seçilen noktadan önceki düşünmeler tutulur, sonrakiler atılır.

Geriye doğru yürümesinin sebebi: erken bir düşünmeyi atmak, arkasındaki her şeyi de bedavaya attırır.
Baştan başlayıp ilk kârlı adayda duran bir tarama öneki gereğinden erken kırar ve sonraki bir
kırılmanın zaten karşılayacağı bir re-prefill'i öder.

### Somut örnek

Kalan bütçe 40K, istek yoğunluğu 70 → ufuk = 40/80 × 70 = **35 istek**. Düşünmeden sonra
bağlamda 15.000 token var.

| Düşünme | Tutmanın maliyeti (35 × 0,2 × düşünme) | Atmanın maliyeti (15.000 × 0,8) | Karar |
|---|---|---|---|
| 32 token | 224 | 12.000 | **tut** |
| 5.000 token | 35.000 | 12.000 | **at** |

Başabaş noktası kabaca `düşünme ≈ 4 × sonraki_kısım / ufuk`; bu örnekte ~1.700 token.

### Önceki promptun ayak izi

Kırılma, *bu prompt ile bir öncekinin* arasındaki farktır; geçmişin tamamıyla değil. Bu yüzden
her karardan sonra hangi öğelerin gönderildiği kaydedilir (`PromptFootprint`) ve bir sonraki
karar ona göre fiyatlanır:

- Önceki promptun zaten dışarıda bıraktığı bir öğe (ör. bitmiş görünmez bir turun isteği) bu
  sefer de dışarıda kalırsa kırılma **değildir**; önbellek onu hiç tutmadı.
- Henüz hiçbir promptun taşımadığı bir öğeyi atmak hiçbir şeyi yeniden faturalamaz; bedavadır.
- Geçmiş bu arada yeniden yazıldıysa (compaction, rollback) eski ayak izi geçersiz sayılır.

### Dondurma

Karar öğenin üstüne yazılır ve bir daha hesaplanmaz. İstekten isteğe fikir değiştiren bir karar,
bu mekanizmanın önlemeye çalıştığı önbellek kaçırmasını kendisi üretirdi. Compaction promptu da
aynı damgalara uyar; böylece özetleme isteği, kendinden önceki isteğin önekini bozmaz.

## Ne yapmaz

- **Aktif turun düşünmesine dokunmaz.** Model bir kullanıcı mesajına cevap verirken yaptığı bütün
  araç döngüsü boyunca kendi düşünmelerini görmeye devam eder. Kırpma yalnızca *önceki*
  kullanıcı turlarının düşünmelerini hedefler. (tabby koşusunda: JOIN turunda model, JOIN turu
  içindeki önceki düşünmelerini görüyordu; görmediği, ORDER BY / LIMIT / HAVING turlarındaki
  düşünmelerdi — o turlarda tutulmaya değer bulunanlar hariç.)
- **Metni kısaltmaz.** Özet çıkarmaz, blokları bölmez; öğe başına tut/at.
- **Araç çıktılarını atmaz.** Bitmiş turların araç çıktıları bağlamda kalır.

## Araç çıktısı kırpmasıyla ilişkisi

`context_manager/tool_output.rs`, bitmiş turlardaki gürültülü build/test çıktısını baş ve son
satırlarına indirir. Bu da çıktıyı yerinde yeniden yazar, yani önbelleği bozar. O yüzden yalnızca
ilk atılan öğeden **sonra** çalışır; önek o noktada zaten kırılmış olduğu için ek bedeli yoktur.
Hiçbir şey atılmadıysa hiçbir şey kısaltılmaz ve önek birebir aynı kalır. Düşünme kırpmasının
verdiği "nereden kıralım" kararı, araç çıktısı kırpmasının da nereden bedava olduğunu belirler.

## Testler

`core/src/reasoning_retention.rs` içindeki testler davranışın sözleşmesi:

| Test | Sabitlediği davranış |
|---|---|
| `tiny_reasoning_is_kept_because_dropping_it_rewrites_the_whole_turn` | 32 token + arkasında 15K → tutulur |
| `large_reasoning_is_dropped_once_it_outweighs_the_reprefill` | 5.000 token + arkasında 15K → atılır |
| `later_reasoning_is_dropped_for_free_once_the_prefix_is_already_broken` | Önek kırıldıktan sonraki küçük düşünme de bedavaya atılır |
| `a_decision_is_never_revisited` | Girdiler değişse de verilmiş karar değişmez |
| `active_turn_reasoning_is_left_undecided` | Aktif turun düşünmesine karar verilmez |
| `an_item_the_previous_prompt_already_left_out_is_not_a_break` | Zaten dışarıda olan öğe kırılma sayılmaz |
| `reasoning_no_prompt_has_carried_yet_is_dropped_for_free` | Hiç gönderilmemiş düşünme bütçe olmasa da bedavaya atılır |
| `unpriced_reasoning_is_dropped_the_moment_its_turn_ends` | Bayrak kapalıyken eski davranış |

## Dosya haritası

| Dosya | İçerik |
|---|---|
| `core/src/reasoning_retention.rs` | Fiyatlama, ufuk, kırılma seçimi, dondurma; testler |
| `core/src/context_manager/history.rs` | `freeze_reasoning_retention` / `drop_completed_reasoning` sarmalayıcıları, `PromptFootprint`, `dropped_reason`, `for_prompt_annotated` |
| `core/src/session/mod.rs` | `reasoning_turn` etiketleme; bayrak ve bütçe kontrolü; `RequestDensity` güncelleme |
| `core/src/session/turn.rs` | Her istekten önce tetikleme noktası |
| `core/src/request_density.rs` | `WINDOW_TOKENS`, pencere başına istek ölçümü |
| `features/src/lib.rs` | `reasoning_cost_model` bayrağı (Stable, varsayılan açık) |
| `history/src/lib.rs` | `reasoning_turn`, `reasoning_retained` metadata alanları |
