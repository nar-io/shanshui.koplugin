#!/bin/sh
# Shan Shui screensaver generator powered by native Rust + tiny-skia/resvg
# Standalone binary with zero external dependencies.

PLUGIN_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN_DIR="${PLUGIN_DIR}/bin"

PID_FILE="/tmp/shanshui_generator.pid"
if [ -f "${PID_FILE}" ]; then
    OLD_PID=$(cat "${PID_FILE}" 2>/dev/null)
    if [ -n "${OLD_PID}" ] && kill -0 "${OLD_PID}" 2>/dev/null; then
        echo "ShanShui: Generator already running with PID ${OLD_PID}"
        exit 0
    fi
fi
echo $$ > "${PID_FILE}"

OUT_DIR="${1:-/mnt/us/koreader/screensavers/shanshui}"
WIDTH="${2:-1236}"
HEIGHT="${3:-1648}"
TRANSPARENT="${4:-0}"

TRANS_FLAG=""
if [ "${TRANSPARENT}" = "1" ] || [ "${TRANSPARENT}" = "true" ]; then
    TRANS_FLAG="--transparent"
fi

# Detect architecture and select compatible binary
choose_binary() {
    ARCH="$(uname -m 2>/dev/null || echo 'armv7l')"
    
    # 64-bit ARM (Onyx Boox, modern Android e-readers, 64-bit Kobo)
    if [ "${ARCH}" = "aarch64" ] && [ -f "${BIN_DIR}/shanshui_aarch64" ]; then
        echo "${BIN_DIR}/shanshui_aarch64"
        return
    fi
    
    # 32-bit ARM with hardware floating point (VFP / NEON) - Modern Kindles & Kobos
    if [ -f "${BIN_DIR}/shanshui_armhf" ]; then
        if grep -qi "vfp" /proc/cpuinfo 2>/dev/null; then
            echo "${BIN_DIR}/shanshui_armhf"
            return
        fi
    fi
    
    # Legacy ARM soft-float - Kindle 3/4/Touch/5/PW1/PW2
    if [ -f "${BIN_DIR}/shanshui_armel" ]; then
        echo "${BIN_DIR}/shanshui_armel"
        return
    fi

    # Fallback to armhf or first existing binary
    if [ -f "${BIN_DIR}/shanshui_armhf" ]; then
        echo "${BIN_DIR}/shanshui_armhf"
    elif [ -f "${BIN_DIR}/shanshui_armv7" ]; then
        echo "${BIN_DIR}/shanshui_armv7"
    else
        echo "${BIN_DIR}/shanshui_armel"
    fi
}

SHAN_BIN="$(choose_binary)"
chmod +x "${BIN_DIR}"/shanshui_* 2>/dev/null

mkdir -p "${OUT_DIR}"
TMP_DIR="${OUT_DIR}/.tmp"
mkdir -p "${TMP_DIR}"

TMP_ID="shan_$$"
TMP_PNG="${TMP_DIR}/${TMP_ID}.png"
FINAL_CURRENT_PNG="${OUT_DIR}/current.png"

# High-entropy seed
SEED="$(date +%s%N 2>/dev/null || date +%s)_$$"

# Execute native binary
"${SHAN_BIN}" "${TMP_PNG}" "${SEED}" "${WIDTH}" "${HEIGHT}" ${TRANS_FLAG} >/dev/null 2>&1

# If execution failed (e.g. SIGILL or missing instruction on older hardware), fallback to legacy binary
if [ ! -s "${TMP_PNG}" ] && [ "${SHAN_BIN}" != "${BIN_DIR}/shanshui_armel" ] && [ -f "${BIN_DIR}/shanshui_armel" ]; then
    echo "ShanShui: Binary ${SHAN_BIN} failed, falling back to legacy armel binary..."
    "${BIN_DIR}/shanshui_armel" "${TMP_PNG}" "${SEED}" "${WIDTH}" "${HEIGHT}" ${TRANS_FLAG} >/dev/null 2>&1
fi

# Atomic update & pool archiving
if [ -s "${TMP_PNG}" ]; then
    STAMP="$(date +%Y%m%d_%H%M%S)"
    cp -f "${TMP_PNG}" "${OUT_DIR}/landscape_${STAMP}.png"
    cp -f "${TMP_PNG}" "${FINAL_CURRENT_PNG}"
    
    if [ -d "/mnt/us/koreader/sleepscreen" ]; then
        cp -f "${TMP_PNG}" "/mnt/us/koreader/sleepscreen/shanshui_current.png"
    fi
    
    rm -f "${TMP_PNG}"

    (cd "${OUT_DIR}" && ls -t landscape_*.png 2>/dev/null | tail -n +6 | xargs rm -f 2>/dev/null)
    echo "ShanShui: Successfully generated landscape (transparent=${TRANSPARENT}) -> ${FINAL_CURRENT_PNG}"
else
    echo "ShanShui: Landscape generation failed"
fi

rm -f "${PID_FILE}"
