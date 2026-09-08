import sqlite3
from pathlib import Path

from db import hash_password, utc_now


DB_PATH = Path(__file__).with_name("sepet.db")
SCHEMA_PATH = Path(__file__).with_name("schema.sql")


RESTAURANTS = [
    {
        "name": "Anadolu Ateşi",
        "cuisine": "kebap",
        "rating": 4.7,
        "delivery_fee": 14.90,
        "min_order": 120.00,
        "eta_minutes": 35,
        "image_emoji": "🍢",
        "menu": [
            ("Adana Kebap Porsiyon", "Acılı kıyma kebap, lavaş, közlenmiş domates ve yeşil biber", 185.00, "Kebaplar"),
            ("Urfa Kebap Porsiyon", "Acısız kıyma kebap, bulgur pilavı ve turşu eşliğinde", 175.00, "Kebaplar"),
            ("Kuzu Şiş", "Marine kuzu eti, közlenmiş sebze ve pirinç pilavı", 225.00, "Kebaplar"),
            ("Beyti Sarma", "Izgara dana kebap, yoğurt ve tereyağlı sos", 210.00, "Kebaplar"),
            ("Humus", "Nohut, tahin, limon ve zeytinyağı ile yumuşak bir başlangıç", 78.00, "Başlangıçlar"),
            ("Künefe", "Tel kadayıf, peynir, tereyağı ve şerbet", 105.00, "Tatlılar"),
        ],
    },
    {
        "name": "Napoli Fırını",
        "cuisine": "pizza",
        "rating": 4.5,
        "delivery_fee": 12.50,
        "min_order": 90.00,
        "eta_minutes": 30,
        "image_emoji": "🍕",
        "menu": [
            ("Margherita", "San Marzano domates, mozzarella ve taze fesleğen", 145.00, "Pizzalar"),
            ("Diavola", "Acılı salam, mozzarella, chilli yağı ve zeytin", 165.00, "Pizzalar"),
            ("Quattro Formaggi", "Mozzarella, gorgonzola, parmesan ve kaşar", 185.00, "Pizzalar"),
            ("Prosciutto E Rucola", "Parma jambonu, roka, parmesan ve balzamik sos", 195.00, "Pizzalar"),
            ("Bruschetta", "Fırınlanmış ekşi maya ekmek, domates, sarımsak ve fesleğen", 68.00, "Başlangıçlar"),
            ("Tiramisu", "Mascarpone, kahve ve kakao ile klasik İtalyan tatlısı", 95.00, "Tatlılar"),
        ],
    },
    {
        "name": "Cadde Burger",
        "cuisine": "burger",
        "rating": 4.4,
        "delivery_fee": 9.90,
        "min_order": 80.00,
        "eta_minutes": 25,
        "image_emoji": "🍔",
        "menu": [
            ("Cadde Classic", "Dana köfte, cheddar, marul, domates ve özel sos", 132.00, "Burgerler"),
            ("Smoky BBQ", "Izgara dana köfte, cheddar, karamelize soğan ve bbq sos", 158.00, "Burgerler"),
            ("Blue Cheese Burger", "Dana köfte, gorgonzola, soğan halkası ve roka", 172.00, "Burgerler"),
            ("Tavuk Crispy", "Çıtır tavuk göğsü, acı mayo, turşu ve marul", 128.00, "Burgerler"),
            ("Patates Kızartması", "Kalın dilim, dışı çıtır patates ve tuz", 48.00, "Yanında"),
            ("Çikolatalı Brownie", "Sıcak çikolatalı brownie ve vanilyalı sos", 72.00, "Tatlılar"),
        ],
    },
    {
        "name": "Şanlı Çiğköfte Evi",
        "cuisine": "çiğ köfte",
        "rating": 4.6,
        "delivery_fee": 7.50,
        "min_order": 55.00,
        "eta_minutes": 20,
        "image_emoji": "🌯",
        "menu": [
            ("Çiğ Köfte Dürüm", "Acılı çiğ köfte, marul, nar ekşisi ve limon", 72.00, "Dürümler"),
            ("Nohutlu Çiğ Köfte Dürüm", "Nohutlu çiğ köfte, yeşillik ve nar ekşisi", 78.00, "Dürümler"),
            ("Çiğ Köfte Servis", "İki kişilik çiğ köfte, lavaş, marul ve limon", 135.00, "Servisler"),
            ("Ayran", "Ev yapımı yoğurttan soğuk ayran", 25.00, "İçecekler"),
            ("Şalgam Suyu", "Acılı veya acısız gelen taze şalgam", 22.00, "İçecekler"),
            ("Şekerpare", "Şerbetli, tereyağlı Şanlıurfa usulü tatlı", 48.00, "Tatlılar"),
        ],
    },
    {
        "name": "Sakura Sushi Bar",
        "cuisine": "sushi",
        "rating": 4.8,
        "delivery_fee": 18.00,
        "min_order": 160.00,
        "eta_minutes": 40,
        "image_emoji": "🍣",
        "menu": [
            ("Salmon Nigiri Seti", "Altı parça taze somon nigiri", 210.00, "Sushiler"),
            ("California Roll", "Yengeç salatası, avokado, salatalık ve susam", 185.00, "Roller"),
            ("Spicy Tuna Roll", "Acı ton balığı, salatalık ve sesame mayo", 198.00, "Roller"),
            ("Dragon Roll", "Karides tempura, avokado ve unagi sos", 225.00, "Roller"),
            ("Miso Çorbası", "Tofu, wakame ve mantarlı miso çorbası", 52.00, "Başlangıçlar"),
            ("Matcha Cheesecake", "Yeşil çaylı, hafif tatlı cheesecake", 98.00, "Tatlılar"),
        ],
    },
    {
        "name": "Baklavacı Memiş",
        "cuisine": "tatlı",
        "rating": 4.9,
        "delivery_fee": 8.00,
        "min_order": 70.00,
        "eta_minutes": 22,
        "image_emoji": "🥮",
        "menu": [
            ("Fıstıklı Baklava", "Antep fıstığı, tereyağı ve ince yufka", 165.00, "Baklava"),
            ("Cevizli Baklava", "Ceviz dolgulu, şerbetli klasik baklava", 145.00, "Baklava"),
            ("Şöbiyet", "Kaymaklı ve fıstıklı şerbetli tatlı", 175.00, "Şerbetliler"),
            ("Sütlaç", "Fırınlanmış, tarçınlı ev sütlacı", 68.00, "Sütlüler"),
            ("Aşure", "Baklagil, kuru meyve ve fındıklı geleneksel aşure", 58.00, "Klasikler"),
            ("Kadınlı Göğsü", "Tavuk göğsü, şerbet ve tarçın ile hafif tatlı", 72.00, "Sütlüler"),
        ],
    },
    {
        "name": "Levent'in Mutfağı",
        "cuisine": "ev yemeği",
        "rating": 4.3,
        "delivery_fee": 11.00,
        "min_order": 85.00,
        "eta_minutes": 28,
        "image_emoji": "🍲",
        "menu": [
            ("Mercimek Çorbası", "Kırmızı mercimek, nane ve limon", 52.00, "Çorbalar"),
            ("Etli Nohut", "Kuzu etli nohut ve bulgur pilavı", 138.00, "Ana Yemekler"),
            ("Karnıyarık", "Kıymalı patlıcan, pirinç pilavı ve cacık", 125.00, "Ana Yemekler"),
            ("Mantı", "Yoğurtlu, naneli ve tereyağlı ev mantısı", 145.00, "Ana Yemekler"),
            ("Zeytinyağlı Yaprak Sarma", "Zeytinyağlı pirinçli sarma ve limon", 82.00, "Zeytinyağlılar"),
            ("Kabak Tatlısı", "Cevizli ve tahinli fırın kabak tatlısı", 64.00, "Tatlılar"),
        ],
    },
    {
        "name": "Karadeniz Balık Evi",
        "cuisine": "deniz ürünleri",
        "rating": 4.2,
        "delivery_fee": 15.00,
        "min_order": 150.00,
        "eta_minutes": 38,
        "image_emoji": "🐟",
        "menu": [
            ("Levrek Izgara", "Marine levrek, mevsim yeşillikleri ve limon", 245.00, "Balıklar"),
            ("Hamsi Tava", "Karadeniz usulü unlu hamsi ve turşu", 185.00, "Balıklar"),
            ("Kalamar Tava", "Çıtır kalamar ve tartar sos", 158.00, "Başlangıçlar"),
            ("Balık Çorbası", "Kemikli balık suyu, sebze ve limon", 82.00, "Çorbalar"),
            ("Mısır Ekmeği", "Fırından sıcak Karadeniz mısır ekmeği", 42.00, "Ekmekler"),
            ("Kabak Mücver", "Kabak, dereotu ve yoğurt soslu mücver", 78.00, "Başlangıçlar"),
        ],
    },
]

RESTAURANT_SLUGS = [
    "anadolu-atesi",
    "napoli-firini",
    "cadde-burger",
    "sanli-cigkofte-evi",
    "sakura-sushi-bar",
    "baklavaci-memis",
    "leventin-mutfagi",
    "karadeniz-balik-evi",
]


def main() -> None:
    DB_PATH.unlink(missing_ok=True)
    with sqlite3.connect(DB_PATH) as conn, open(SCHEMA_PATH, encoding="utf-8") as schema_file:
        conn.row_factory = sqlite3.Row
        conn.executescript(schema_file.read())
        for restaurant in RESTAURANTS:
            cursor = conn.execute(
                """INSERT INTO restaurant
                   (name, cuisine, rating, delivery_fee, min_order, eta_minutes, image_emoji)
                   VALUES (?, ?, ?, ?, ?, ?, ?)""",
                (
                    restaurant["name"],
                    restaurant["cuisine"],
                    restaurant["rating"],
                    restaurant["delivery_fee"],
                    restaurant["min_order"],
                    restaurant["eta_minutes"],
                    restaurant["image_emoji"],
                ),
            )
            restaurant_id = cursor.lastrowid
            conn.executemany(
                """INSERT INTO menu_item
                   (restaurant_id, name, description, price, category)
                   VALUES (?, ?, ?, ?, ?)""",
                [
                    (restaurant_id, item[0], item[1], item[2], item[3])
                    for item in restaurant["menu"]
                ],
            )

        password_hash = hash_password("sepet123")
        now = utc_now()
        customer_cursor = conn.execute(
            'INSERT INTO "user" (email, password_hash, role, balance) VALUES (?, ?, ?, ?)',
            ("musteri@sepet.test", password_hash, "customer", 250.0),
        )

        for restaurant, slug in zip(RESTAURANTS, RESTAURANT_SLUGS):
            row = conn.execute(
                "SELECT id FROM restaurant WHERE name = ?", (restaurant["name"],)
            ).fetchone()
            if not row:
                continue
            restaurant_id = row["id"]
            owner_cursor = conn.execute(
                'INSERT INTO "user" (email, password_hash, role, balance) VALUES (?, ?, ?, ?)',
                (f"sahip.{slug}@sepet.test", password_hash, "restaurant", 0.0),
            )
            conn.execute(
                "UPDATE restaurant SET user_id = ? WHERE id = ?",
                (owner_cursor.lastrowid, restaurant_id),
            )
            for number in (1, 2):
                courier_user = conn.execute(
                    'INSERT INTO "user" (email, password_hash, role, balance) VALUES (?, ?, ?, ?)',
                    (f"kurye{number}.{slug}@sepet.test", password_hash, "courier", 0.0),
                )
                conn.execute(
                    "INSERT INTO courier (user_id, restaurant_id, name) VALUES (?, ?, ?)",
                    (
                        courier_user.lastrowid,
                        restaurant_id,
                        f"{restaurant['name']} Kurye {number}",
                    ),
                )

        print("Demo musteri: musteri@sepet.test / sepet123")
        print(f"{len(RESTAURANTS)} restoran ve veritabani olusturuldu: {DB_PATH}")


if __name__ == "__main__":
    main()
