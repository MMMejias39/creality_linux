#!/bin/bash
# Launcher do Creality Print com correção de locale.
# O AppRun original do AppImage força LC_ALL=C; com isso o app escolhe o idioma
# en_GB, que não existe neste sistema, exibe erro e encerra. Aqui usamos o
# locale real do sistema (pt_BR.UTF-8), que permite a inicialização normal.
DIR="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)/CrealityPrint-7.3.0.AppDir"
export LD_LIBRARY_PATH="$DIR/bin:$DIR/usr/lib"
export LC_ALL="pt_BR.UTF-8"

# Workaround GLX para sistemas híbridos Intel/AMD + NVIDIA (igual ao AppRun):
# preferir Mesa quando o VGA de boot não é NVIDIA.
if [ -z "${__GLX_VENDOR_LIBRARY_NAME+x}" ]; then
    have_nvidia_glx=0
    if command -v ldconfig >/dev/null 2>&1 &&
       ldconfig -p 2>/dev/null | grep -q 'libGLX_nvidia\.so\.0'; then
        have_nvidia_glx=1
    else
        for glx_library in /usr/lib/*/libGLX_nvidia.so.0 /usr/lib*/libGLX_nvidia.so.0 /lib/*/libGLX_nvidia.so.0 /lib*/libGLX_nvidia.so.0; do
            if [ -e "$glx_library" ]; then
                have_nvidia_glx=1
                break
            fi
        done
    fi

    if [ "$have_nvidia_glx" = "1" ]; then
        for drm_device in /sys/class/drm/card*/device; do
            boot_vga=$(cat "$drm_device/boot_vga" 2>/dev/null)
            gpu_vendor=$(cat "$drm_device/vendor" 2>/dev/null)
            if [ "$boot_vga" = "1" ] && [ "$gpu_vendor" != "0x10de" ]; then
                export __GLX_VENDOR_LIBRARY_NAME=mesa
                break
            fi
        done
    fi
fi

exec "$DIR/bin/CrealityPrint" "$@"
