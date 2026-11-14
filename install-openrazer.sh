#!/usr/bin/env bash
# Install the OpenRazer driver + daemon on Fedora (handles Secure Boot module signing).
# Other distributions: https://openrazer.github.io/#download
set -euo pipefail

FEDORA=$(rpm -E %fedora)
KERNEL=$(uname -r)
REPO_URL="https://download.opensuse.org/repositories/hardware:/razer/Fedora_${FEDORA}/hardware:razer.repo"

echo "==> Fedora ${FEDORA}, kernel ${KERNEL}"
if ! curl -fsSL -o /dev/null "$REPO_URL"; then
  echo "!! No OpenRazer repository for Fedora_${FEDORA} yet: $REPO_URL"
  echo "   See https://openrazer.github.io/#download"
  exit 1
fi

echo "==> kernel-devel + dkms"
sudo dnf install -y "kernel-devel-${KERNEL}" dkms
echo "==> OpenRazer repository + packages"
sudo dnf config-manager addrepo --overwrite --from-repofile="$REPO_URL"
sudo dnf install -y openrazer-meta
echo "==> Adding $USER to the plugdev group"
sudo gpasswd -a "$USER" plugdev

echo "==> DKMS status:"
sudo dkms status | grep -i razer || true
if ! sudo dkms status | grep -i openrazer | grep -q "${KERNEL}.*installed"; then
  echo "!! The OpenRazer module was not built for kernel ${KERNEL}."
  echo "   Log: ls /var/lib/dkms/openrazer-driver/*/build/make.log"
  exit 1
fi

# Secure Boot: enroll the DKMS signing key (MOK)
if mokutil --sb-state 2>/dev/null | grep -q enabled; then
  MOK_PUB=/var/lib/dkms/mok.pub
  if [ -f "$MOK_PUB" ]; then
    if sudo mokutil --test-key "$MOK_PUB" 2>&1 | grep -q "already enrolled"; then
      echo "==> DKMS key already enrolled."
    else
      echo
      echo "==> Secure Boot: enrolling the DKMS key. Choose a one-time password (e.g. 1234)."
      sudo mokutil --import "$MOK_PUB"
      echo "    AFTER REBOOT a blue 'MOK management' screen appears:"
      echo "    Enroll MOK -> Continue -> Yes -> enter that password -> Reboot"
    fi
  else
    echo "!! $MOK_PUB not found – the module may not be signed."
  fi
fi
echo
echo "==> Done. Reboot your computer."
