#!/usr/bin/env bash
# One-time / idempotent host setup for the EarDo ARM VM (Ubuntu 24.04, aarch64).
# Run from a checkout of this repo on the VM (or after copying deploy/ there):
#   sudo bash deploy/bootstrap/setup-vm.sh
# Safe to re-run: it only (re)installs scripts, units, Caddy config and
# directories, and never overwrites existing /etc/eardo/*.env secrets.
set -euo pipefail
[[ $EUID -eq 0 ]] || { echo "run with sudo" >&2; exit 1; }
HERE=$(cd "$(dirname "$0")/.." && pwd)

echo "== packages"
need=()
for p in caddy postgresql-client-16 jq curl; do dpkg -s "$p" >/dev/null 2>&1 || need+=("$p"); done
if ((${#need[@]})); then apt-get update -q && DEBIAN_FRONTEND=noninteractive apt-get install -yq "${need[@]}"; fi

echo "== accounts"
for u in eardo develop; do
  id "$u" >/dev/null 2>&1 || useradd --create-home --shell /bin/bash "$u"
done

echo "== directories"
install -d -m 0755 -o root -g root /opt/eardo /opt/eardo-dev /etc/eardo
for b in /opt/eardo /opt/eardo-dev; do
  install -d -m 0755 -o root -g root "$b/releases" "$b/slots" "$b/state"
done
install -d -m 0750 -o eardo -g eardo /opt/eardo/incoming
install -d -m 0750 -o develop -g develop /opt/eardo-dev/incoming

echo "== scripts"
install -m 0755 -o root -g root "$HERE/bin/eardo-rollout" /usr/local/sbin/eardo-rollout
install -m 0755 -o root -g root "$HERE/bin/eardo-deploy-gate" /usr/local/bin/eardo-deploy-gate

echo "== sudoers (each account may only manage its own environment)"
cat >/etc/sudoers.d/eardo-deploy <<'EOF'
# Managed by eardo deploy/bootstrap/setup-vm.sh
eardo   ALL=(root) NOPASSWD: /usr/local/sbin/eardo-rollout prod *
develop ALL=(root) NOPASSWD: /usr/local/sbin/eardo-rollout dev *
EOF
chmod 0440 /etc/sudoers.d/eardo-deploy
visudo -cf /etc/sudoers.d/eardo-deploy >/dev/null

echo "== env files"
for e in prod dev; do
  f=/etc/eardo/$e.env
  if [[ ! -f $f ]]; then
    install -m 0600 -o root -g root /dev/null "$f"
    if [[ $e == prod ]]; then o=https://eardoai.com,https://www.eardoai.com; else o=https://dev.eardoai.com; fi
    cat >"$f" <<EOF
# EarDo $e runtime environment (root-only, read by systemd). KEY=VALUE, no spaces.
PG_DATABASE_URL=
ALIYUN_API_KEY=
OPENROUTER_API_KEY=
ALLOWED_ORIGINS=$o
COOKIE_SECURE=true
PG_MAX_CONNECTIONS=5
TTS_MAX_CONCURRENCY=2
SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
EOF
    echo "   created $f — fill in secrets!"
  fi
done
printf 'LEPTOS_SITE_ADDR=127.0.0.1:3000\n' >/etc/eardo/prod-slot-a.env
printf 'LEPTOS_SITE_ADDR=127.0.0.1:3001\n' >/etc/eardo/prod-slot-b.env
printf 'LEPTOS_SITE_ADDR=127.0.0.1:3100\n' >/etc/eardo/dev-slot-a.env
printf 'LEPTOS_SITE_ADDR=127.0.0.1:3101\n' >/etc/eardo/dev-slot-b.env
chmod 0644 /etc/eardo/*-slot-*.env

echo "== systemd units"
install -m 0644 "$HERE/systemd/eardo-prod@.service" /etc/systemd/system/
install -m 0644 "$HERE/systemd/eardo-dev@.service" /etc/systemd/system/
install -m 0644 "$HERE/systemd/eardo-cleanup.service" "$HERE/systemd/eardo-cleanup.timer" /etc/systemd/system/
systemctl daemon-reload
systemctl enable -q --now eardo-cleanup.timer

echo "== caddy"
install -d -m 0750 -o root -g caddy /etc/caddy/certs
if [[ ! -f /etc/caddy/certs/origin.pem ]]; then
  echo "   WARNING: put the Cloudflare Origin CA cert/key at /etc/caddy/certs/origin.{pem,key} (root:caddy 0640)"
fi
install -m 0644 "$HERE/caddy/Caddyfile" /etc/caddy/Caddyfile
if [[ -f /etc/caddy/certs/origin.pem ]]; then
  caddy validate --adapter caddyfile --config /etc/caddy/Caddyfile >/dev/null
  systemctl reload caddy || systemctl restart caddy
fi
echo "done. Next: fill /etc/eardo/*.env, add CI keys (see docs/handover.md), deploy a release."
