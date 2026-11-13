#!/bin/sh
# Shan Shui screensaver generator powered by native Rust + tiny-skia/resvg
# Standalone binary with zero external dependencies.
#
# Usage: generator.sh <out_dir> <width> <height> <transparent> [modes]
#   modes: comma-separated subset of "portrait,landscape" (default: both)
#
# Each orientation gets its own slot (portrait.png / landscape.png) so the
# screensaver can print the picture that matches the screen formation it is
# waking into, instead of whatever orientation was generated last.

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
MODES="${5:-portrait,landscape}"

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

# WIDTH/HEIGHT describe the portrait canvas; the landscape slot is the same
# canvas rotated, so both slots share one device profile.
LS_WIDTH="${HEIGHT}"
LS_HEIGHT="${WIDTH}"

has_mode() {
    case ",${MODES}," in
        *",$1,"*) return 0 ;;
        *) return 1 ;;
    esac
}

# Render one orientation into its slot. Args: <suffix> <width> <height>
render_slot() {
    SUFFIX="$1"
    W="$2"
    H="$3"

    TMP_PNG="${TMP_DIR}/shan_$$_${SUFFIX}.png"
    SEED="$(date +%s%N 2>/dev/null || date +%s)_$$_${SUFFIX}"

    "${SHAN_BIN}" "${TMP_PNG}" "${SEED}" "${W}" "${H}" ${TRANS_FLAG} >/dev/null 2>&1

    # If execution failed (e.g. SIGILL or missing instruction on older
    # hardware), fall back to the legacy armel binary.
    if [ ! -s "${TMP_PNG}" ] && [ "${SHAN_BIN}" != "${BIN_DIR}/shanshui_armel" ] && [ -f "${BIN_DIR}/shanshui_armel" ]; then
        echo "ShanShui: Binary ${SHAN_BIN} failed, falling back to legacy armel binary..."
        "${BIN_DIR}/shanshui_armel" "${TMP_PNG}" "${SEED}" "${W}" "${H}" ${TRANS_FLAG} >/dev/null 2>&1
    fi

    if [ -s "${TMP_PNG}" ]; then
        STAMP="$(date +%Y%m%d_%H%M%S)"
        # Pool copy keeps a few recent pictures per orientation for browsing.
        cp -f "${TMP_PNG}" "${OUT_DIR}/landscape_${SUFFIX}_${STAMP}.png"
        # The slot the screensaver reads. Written via a temp file + rename so a
        # sleep that happens mid-write never picks up a half-written PNG.
        cp -f "${TMP_PNG}" "${TMP_DIR}/${SUFFIX}.png.part"
        mv -f "${TMP_DIR}/${SUFFIX}.png.part" "${OUT_DIR}/${SUFFIX}.png"
        rm -f "${TMP_PNG}"
        echo "ShanShui: generated ${SUFFIX} (${W}x${H}, transparent=${TRANSPARENT})"
        return 0
    fi
    rm -f "${TMP_PNG}"
    echo "ShanShui: ${SUFFIX} generation failed"
    return 1
}

OK=0
if has_mode portrait; then
    render_slot portrait "${WIDTH}" "${HEIGHT}" && OK=1
fi
if has_mode landscape; then
    render_slot landscape "${LS_WIDTH}" "${LS_HEIGHT}" && OK=1
fi

# Keep the newest few pool entries per orientation.
(cd "${OUT_DIR}" && ls -t landscape_portrait_*.png 2>/dev/null | tail -n +4 | xargs rm -f 2>/dev/null)
(cd "${OUT_DIR}" && ls -t landscape_landscape_*.png 2>/dev/null | tail -n +4 | xargs rm -f 2>/dev/null)

if [ "${OK}" = "1" ]; then
    echo "ShanShui: slots ready in ${OUT_DIR}"
else
    echo "ShanShui: Landscape generation failed"
fi

rm -f "${PID_FILE}"
