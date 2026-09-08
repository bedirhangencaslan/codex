# Sepet

FastAPI + SQLite backend ve Vite + React + TypeScript frontend ile hazirlanmis,
odemesiz bir yemek siparis ornegi. Hesap girisi, restoran ve kurye panelleri,
siparis durumu ile musteri onayi ve restoran yorumlari icerir.

## Calistirma

### 1. Backend

Yeni bir terminal acin:

`powershell
cd api
python seed.py
python -m uvicorn main:app --port 8000
`

Backend http://127.0.0.1:8000 adresinde calisir.

### 2. Frontend

Yeni bir terminal acin:

`powershell
cd web
npm install
npm run dev
`

Frontend varsayilan olarak http://127.0.0.1:5173 adresinde calisir. 5173 portu
doluysa Vite genellikle 5174 portunu onerir. /api istekleri Vite tarafindan
http://127.0.0.1:8000 adresine yonlendirilir.

## Demo Hesaplar

Seed verisindeki tum hesaplarin sifresi: sepet123

| Rol | E-posta |
| --- | --- |
| Musteri | musteri@sepet.test |
| Restoran | sahip.kebapci@sepet.test, sahip.napoli@sepet.test vb. |
| Kurye | kurye1.kebapci@sepet.test, kurye2.kebapci@sepet.test vb. |

Restoranlar Giris / Kayit sayfasindaki listeden de secilebilir.

## Siparis Akisi

1. Musteri siparis olusturur; siparis durumu hazirlaniyor olur.
2. Restoran panelinden siparisi kendi kuryelerinden birine atar.
3. Kurye panelinde siparisi teslim edildi olarak isaretler.
4. Musteri siparisi onaylar.
5. Onaylanan siparis icin musteri puan ve yorum birakabilir.

## Onemli Not

python seed.py komutu varsa api/sepet.db dosyasini siler ve veritabanini
sifirdan kurar. Yeni hesaplarla yaptiginiz degisiklikler de sifirlanir.
