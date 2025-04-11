# package_game.ps1

$projectName = "Poligon"
$buildType = "release"
$targetDir = "target/$buildType"
$distDir = "dist/$projectName"
$zipFile = "$projectName.zip"

Write-Host "Derleniyor..."
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "[Error] Derleme başarısız!" -ForegroundColor Red
    exit 1
}

# Temizlik ve klasör hazırlığı
if (Test-Path $distDir) {
    Remove-Item $distDir -Recurse -Force
}
New-Item -ItemType Directory -Path $distDir | Out-Null

# .exe dosyasını kopyala
Copy-Item "$targetDir/poligon.exe" "$distDir/poligon.exe"

# assets klasörünü kopyala
Copy-Item "assets" "$distDir/assets" -Recurse

# Daha önce varsa eski zip'i sil
if (Test-Path $zipFile) {
    Remove-Item $zipFile -Force
}

# Zip oluştur
Write-Host "Zip dosyası hazırlanıyor..."
Compress-Archive -Path "$distDir/*" -DestinationPath $zipFile

Write-Host "[Ok] Paket hazır: $zipFile" -ForegroundColor Green
