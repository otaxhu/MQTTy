#!/bin/bash

# Copyright (c) 2025 Oscar Pernia
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

# This script builds the app, vendorizes the runtime files (DLLs, i18n files, icons, etc.)
# and compresses the folder into a zip file, resulting into which is known in
# the Windows ecosystem as a "portable application"

set -e

USAGE="
Usage: $0 [options]

options:
    -p (default|development)  profile option passed to Meson
"

pacman -Sy --noconfirm --needed \
    ${MINGW_PACKAGE_PREFIX}-desktop-file-utils \
    ${MINGW_PACKAGE_PREFIX}-gcc \
    ${MINGW_PACKAGE_PREFIX}-cmake \
    ${MINGW_PACKAGE_PREFIX}-gettext-tools \
    ${MINGW_PACKAGE_PREFIX}-gtk4 \
    ${MINGW_PACKAGE_PREFIX}-gtksourceview5 \
    ${MINGW_PACKAGE_PREFIX}-meson \
    ${MINGW_PACKAGE_PREFIX}-make \
    ${MINGW_PACKAGE_PREFIX}-pkgconf \
    ${MINGW_PACKAGE_PREFIX}-libadwaita \
    ${MINGW_PACKAGE_PREFIX}-blueprint-compiler \
    zip

PROFILE=default

while getopts "p:" opt; do
    case $opt in
        p)
            PROFILE=$OPTARG
            if [[ $PROFILE != default && $PROFILE != development ]]; then
                echo "-p option must one of 'default' or 'development'" 1>&2
                echo "$USAGE" 1>&2
                exit 1
            fi
            ;;
        ? | :)
            echo "$USAGE" 1>&2
            exit 1
            ;;
    esac
done

ROOT_DIR=$PWD

MSYS2_ARG_CONV_EXCL="--prefix=" meson setup build --prefix=/ -Dprofile=$PROFILE

rm -rf $ROOT_DIR/build/win32-portable

# FIXME: Sometimes the command works, sometimes not,
# It fails during Paho MQTT C library building, but if you try one or two
# more tries it just works magically
ninja -C build
DESTDIR=$ROOT_DIR/build/win32-portable ninja -C build install

cd $ROOT_DIR/build/win32-portable

mkdir -p lib share

# Copy required DLLs into bin/
cp $(
    ldd bin/MQTTy.exe $MINGW_PREFIX/gdk-pixbuf-2.0/2.10.0/loaders/*.dll |
    grep "$MINGW_PREFIX" |
    awk '{ print $3 }' |
    sort | uniq
) bin/

cp $MINGW_PREFIX/bin/gdbus.exe bin/
cp $MINGW_PREFIX/bin/gspawn-win64-helper.exe bin/

# MSYS2 delivers .a files, which only contains function declarations.
# These are useful when you are in the compilation step and if you are using C.
#
# At this point the app is already built and we don't care about these files
#
# It could tempting to use -iname '*.dll', but we also need to copy the
# loaders.cache file
PIXBUF_FILES=$(find $MINGW_PREFIX/lib/gdk-pixbuf-2.0/ -type f -not -iname '*.dll.a')

mkdir -p $(dirname $(realpath -m --relative-to=$MINGW_PREFIX $PIXBUF_FILES))

for path in $PIXBUF_FILES; do
    cp $path $(realpath -m --relative-to=$MINGW_PREFIX $path)
done

cp -RTn $MINGW_PREFIX/share/glib-2.0 share/glib-2.0
cp -RTn $MINGW_PREFIX/share/icons/Adwaita share/icons/Adwaita
cp -RTn $MINGW_PREFIX/share/icons/hicolor share/icons/hicolor
cp -RTn $MINGW_PREFIX/share/gtksourceview-5 share/gtksourceview-5

for lang in $(cat "$ROOT_DIR/po/LINGUAS" | grep -E '.+'); do
    for pkg in gdk-pixbuf gettext-runtime glib20 gtk40 gtksourceview-5 libadwaita shared-mime-info; do
        MO_PATH=$MINGW_PREFIX/share/locale/$lang/LC_MESSAGES/$pkg.mo
        if [ -f $MO_PATH ]; then
            cp -f $MO_PATH share/locale/$lang/LC_MESSAGES
        fi
    done
done

glib-compile-schemas.exe share/glib-2.0/schemas
gtk4-update-icon-cache.exe -t share/icons/hicolor

OUTFILE=MQTTy-win32-portable-x86_64.zip

rm -f ../$OUTFILE

zip -r ../$OUTFILE *
