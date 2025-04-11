@echo off
echo Derleniyor: release modunda...
cargo build --release

echo Eski zip dosyasi temizleniyor...
del /Q poligon.zip 2>nul

echo Yeni zip paketi hazirlaniyor...
powershell -Command "Compress-Archive -Path 'target/release/poligon.exe','assets' -DestinationPath 'poligon.zip'"

echo Tamamlandi! poligon.zip hazir.
pause
