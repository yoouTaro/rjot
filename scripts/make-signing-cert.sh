#!/bin/sh
# Create a self-signed code-signing certificate ("rjot Local Signing") in the
# login keychain. bundle-macos.sh signs with it when present. Unlike an ad-hoc
# signature it stays the same across builds, so the Accessibility permission
# survives rebuilding. Remove it with:
#   security delete-identity -c "rjot Local Signing"
set -e
name="rjot Local Signing"
if security find-certificate -c "$name" >/dev/null 2>&1; then
  echo "\"$name\" already exists"; exit 0
fi
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
openssl req -x509 -newkey rsa:2048 -nodes -days 3650 \
  -keyout "$tmp/key.pem" -out "$tmp/cert.pem" -subj "/CN=$name" \
  -addext "basicConstraints=critical,CA:false" \
  -addext "keyUsage=critical,digitalSignature" \
  -addext "extendedKeyUsage=critical,codeSigning" 2>/dev/null
# -legacy: macOS's importer doesn't read OpenSSL 3's default PKCS#12 ciphers.
openssl pkcs12 -export -legacy -inkey "$tmp/key.pem" -in "$tmp/cert.pem" \
  -out "$tmp/id.p12" -passout pass:rjot -name "$name"
security import "$tmp/id.p12" -k "$HOME/Library/Keychains/login.keychain-db" \
  -P rjot -T /usr/bin/codesign >/dev/null
echo "Created \"$name\" in the login keychain"
