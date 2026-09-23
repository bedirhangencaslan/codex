/** Türkçe — el yazması, referans sözlük. Anahtar kümesinin kaynağı bu dosya:
 * diğer diller `Record<MessageKey, string>` ile buna karşı tip-denetlenir,
 * eksik anahtar derleme hatasıdır. Noktalı/noktasız i elle doğrulanır. */
export const tr = {
  "locale.name": "Türkçe",

  "nav.sessions": "Oturumlar",
  "nav.learn": "Eğitim",
  "nav.skills": "Beceriler",
  "nav.commands": "Komutlar",
  "nav.styles": "Stiller",
  "nav.aria": "Ana gezinme",
  "nav.language": "Arayüz dili",

  "sessions.search": "Oturumlarda ara…",
  "sessions.new": "Yeni oturum",
  "sessions.untitled": "(adsız oturum)",
  "sessions.week": "bu hafta",
  "sessions.avgCached": "ort. önbellekli",
  "sessions.fresh": "taze token",
  "sessions.count": "oturum",
  "sessions.cacheNow": "önbellek · 420 sn TTL",
  "sessions.requests": "istek",
  "sessions.compactions": "sıkıştırma",

  "warmth.instant": "ANINDA",
  "warmth.fast": "HIZLI",
  "warmth.cold": "SOĞUK",

  "rail.selected": "Seçili Oturum",
  "rail.cachedInput": "ÖNBELLEKLİ GİRDİ",
  "rail.timeline": "İstek Zaman Çizelgesi",
  "rail.summary": "Özet",
  "rail.fresh": "taze",
  "rail.cached": "önbellekli",
  "rail.output": "çıktı",
  "rail.keepAlive": "canlı tutma",
  "gauge.cachedInput": "Önbellekli girdi",

  "skills.title": "Beceriler",
  "skills.subtitle": "prompt'a giren her blok ölçülür",
  "skills.promptLoad": "Prompt yükü",
  "skills.lighter": "hafifledi",
  "skills.tokens": "token",
  "skills.on": "açık",
  "skills.off": "kapalı",

  "commands.title": "Komutlar",
  "commands.subtitle": "güvenli denemeler geçici oturumda çalışır, geçmişe yazılmaz",
  "commands.try": "Dene",
  "commands.trialIdle": "Deneme alanı",
  "commands.trialHint": "Soldan bir komut seçin; geçici oturumda çalışır, geçmişe yazılmaz.",

  "styles.title": "Stiller",
  "styles.subtitle": "stil seçimi yalnız görünümü değiştirir; oturum verisine dokunmaz",
  "styles.apply": "Uygula",
  "styles.active": "Etkin",
  "styles.thermal.name": "Termal Enstrüman Pro",
  "styles.thermal.tagline": "önbellek sıcaklığı arayüzün fiziği",
  "styles.blueprint.name": "Blueprint Defteri",
  "styles.blueprint.tagline": "milimetrik kağıt üstünde ölçüm günlüğü",
  "styles.abyss.name": "Abis Terminali",
  "styles.abyss.tagline": "çift fosforlu OLED terminal",

  "common.loading": "Yükleniyor…",
  "common.empty": "Henüz içerik yok",
  "common.mockBadge": "örnek veri",
  "common.liveBadge": "canlı: app-server",
  "common.trialFailed": "Deneme tamamlanamadı",

  "learn.title": "Eğitim",
  "learn.subtitle": "kısa dersler; her deneme geçici oturumda çalışır, geçmişe yazılmaz",
  "learn.progress": "ilerleme",
  "learn.lesson": "Ders",
  "learn.prev": "Önceki",
  "learn.next": "Sonraki",
  "learn.markDone": "Tamamlandı işaretle",
  "learn.done": "Tamamlandı",
  "learn.reset": "İlerlemeyi sıfırla",
  "learn.tryIt": "Şimdi dene",
  "learn.tip": "İpucu",
  "learn.backToModules": "Modüllere dön",

  "learn.slash.title": "/ Komutları",
  "learn.slash.desc": "Oturumu yöneten komut dilini altı kısa derste öğrenin — durumdan bağlam yönetimine.",

  "learn.slash.intro.title": "/ komutu nedir?",
  "learn.slash.intro.body":
    "Yazı alanına / yazdığınızda komut listesi açılır. Bu komutlar modele gönderilen bir mesaj değildir; oturumun kendisini yönetir: durumu gösterir, geçmişi özetler, oturumu çatallar. Yani bir / komutu çalıştırmak prompt'a token eklemez — bu, maliyet açısından önemli bir ayrımdır.",
  "learn.slash.intro.tip": "Komut adını tam hatırlamıyorsanız / yazıp birkaç harf yazın; liste süzülür.",

  "learn.slash.status.title": "Durumu okuyun: /status",
  "learn.slash.status.body":
    "/status oturumun kimliğini tek bakışta verir: hangi model, hangi sağlayıcı, önbellek sıcaklığı ve token sayaçları. Taze/önbellekli ayrımı doğrudan faturadır: önbellekli token, tazenin beşte biri fiyatına işlenir. Bir oturum yavaş veya pahalı geliyorsa ilk bakılacak yer burasıdır.",
  "learn.slash.status.tip": "Sağ alttaki hız göstergesi (⚡ Anında / ● Hızlı / ◌ Isınıyor) aynı veriyi canlı özetler.",

  "learn.slash.context.title": "Bağlamı yönetin: /compact ve /recap",
  "learn.slash.context.body":
    "Konuşma uzadıkça bağlam penceresi dolar. /compact geçmişi tek istekte yoğun bir özete indirir — ama bedava değildir: ölçümlerde bir sıkıştırma ~73 bin token'lık soğuk bir isteğe mal oldu. Bu yüzden 80k eşiğindeki otomatik sıkıştırmaya güvenin; /compact'ı yalnızca konu bilinçli olarak değiştiğinde elle çalıştırın. /recap ise geçmişi silmeden yalnızca bir özet yazar.",
  "learn.slash.context.tip": "Sıkıştırma araç çıktısını da özetler; sonrasında model bir dosyayı yeniden okuyabilir. Bu normaldir.",

  "learn.slash.sessions.title": "Oturum yaşam döngüsü: /new, /resume, /fork",
  "learn.slash.sessions.body":
    "/new temiz bir sayfa açar, /resume kayıtlı bir oturumu kaldığı yerden sürdürür. /fork ise en az bilinen ama en güçlüsüdür: geçmişin kopyasıyla yeni bir oturum açar — riskli bir deneme yapmak istediğinizde ana oturumunuz bozulmadan kalır.",
  "learn.slash.sessions.tip": "Uzun bir oturumda yön değiştirecekseniz /fork + /compact ikilisi en ucuz yoldur.",

  "learn.slash.skills.title": "Becerileri çağırın: /skills ve $ad",
  "learn.slash.skills.body":
    "/skills bu projede açık becerileri listeler. Bir beceriyi mesaj içinde $beceri-adi ile çağırırsınız. Dikkat: açık her beceri, talimat bloğunu her isteğin prompt'una ekler — bu ölçülü bir maliyettir. Beceriler ekranı her becerinin kaç token eklediğini gösterir; kullanmadıklarınızı kapatın.",
  "learn.slash.skills.tip": "Bu projenin kendi ölçümü: alakasız beceri bloğunu kapatmak, okuma disiplinini bozan tek değişkendi.",

  "learn.slash.safety.title": "Güvenli deneme: geçici oturumlar",
  "learn.slash.safety.body":
    "Bu ekrandaki her \"Şimdi dene\" düğmesi komutu geçici (ephemeral) bir oturumda çalıştırır: geçmişe yazılmaz, oturum listenizde görünmez, kapanınca yok olur. Yani burada kırabileceğiniz hiçbir şey yok — komutları çekinmeden deneyin, çıktıların şeklini tanıyın.",
  "learn.slash.safety.tip": "Aynı güvence Komutlar ekranındaki deneme alanı için de geçerlidir.",
} as const;
