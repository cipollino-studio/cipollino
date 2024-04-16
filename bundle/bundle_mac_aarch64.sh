
cargo build --release --target aarch64-apple-darwin

rm -Rf mac_aarch64_bundle
mkdir mac_aarch64_bundle

DIR=mac_aarch64_bundle/Cipollino.app

mkdir $DIR
mkdir $DIR/Contents
cp bundle/Info.plist $DIR/Contents/
mkdir $DIR/Contents/MacOS
cp target/aarch64-apple-darwin/release/cipollino $DIR/Contents/MacOS/
mkdir $DIR/Contents/Resources
cp res/icon.icns $DIR/Contents/Resources/Cipollino.icns
cp libs/bin/macos_arm64/ffmpeg $DIR/Contents/ffmpeg