set -e

cargo install --path ~/Documents/HippoWM/

XEPHYR=$(whereis -b Xephyr | cut -f2 -d' ')
xinit ./xinitrc -- \
    "$XEPHYR" \
        :100 \
        -ac \
        -screen 1024x576 \
        -host-cursor
