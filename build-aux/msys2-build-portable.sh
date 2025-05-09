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
    ${MINGW_PACKAGE_PREFIX}-jq \
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

VERSION=$(meson introspect build --projectinfo | jq -r '.version')

OUTDIR=$ROOT_DIR/build/MQTTy-$VERSION-win32-portable-x86_64

rm -rf $OUTDIR

# FIXME: Sometimes the command works, sometimes not,
# It fails during Paho MQTT C library building, but if you try one or two
# more tries it just works magically
ninja -C build
DESTDIR=$OUTDIR ninja -C build install

cd $OUTDIR

mkdir -p lib share

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

# Copy required DLLs into bin/
cp $(
    ldd bin/MQTTy.exe lib/gdk-pixbuf-2.0/2.10.0/loaders/*.dll |
    grep "$MINGW_PREFIX" |
    awk '{ print $3 }' |
    sort | uniq
) bin/

cp -RTn $MINGW_PREFIX/share/glib-2.0 share/glib-2.0
cp -RTn $MINGW_PREFIX/share/icons/Adwaita share/icons/Adwaita
cp -RTn $MINGW_PREFIX/share/icons/hicolor share/icons/hicolor
cp -RTn $MINGW_PREFIX/share/gtksourceview-5 share/gtksourceview-5

for lang in $(cat "$ROOT_DIR/po/LINGUAS"); do
    for pkg in gdk-pixbuf gettext-runtime glib20 gtk40 gtksourceview-5 libadwaita shared-mime-info; do
        MO_PATH=$MINGW_PREFIX/share/locale/$lang/LC_MESSAGES/$pkg.mo
        if [ -f $MO_PATH ]; then
            cp -f $MO_PATH share/locale/$lang/LC_MESSAGES
        fi
    done
done

glib-compile-schemas.exe share/glib-2.0/schemas
gtk4-update-icon-cache.exe -t share/icons/hicolor

# Copy legal files
cp $ROOT_DIR/COPYING $ROOT_DIR/NOTICE .

# Little custom README file
echo "Copyright (c) 2025 Oscar Pernia

This software is licensed under the terms of the GNU GPL 3.0 license,
or later versions. You will find a copy of the license in the COPYING file.

You can download a copy of this software by visiting:

https://github.com/otaxhu/MQTTy/releases

To run MQTTy just execute the bin/MQTTy.exe file" > README

OUTFILE=$(basename $OUTDIR).zip

cd ..

rm -f $OUTFILE

zip -rq $OUTFILE $OUTDIR
