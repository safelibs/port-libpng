#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

IMAGE_TAG="libpng-original-smoke:latest"

required_dependents=(
  "GIMP"
  "LibreOffice Draw"
  "Scribus"
  "WebKitGTK"
  "GDK Pixbuf"
  "Cairo"
  "SDL2_image"
  "feh"
  "Netpbm"
  "XSane"
  "R png package"
  "pngquant"
)

for tool in docker jq; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    printf 'missing required host tool: %s\n' "$tool" >&2
    exit 1
  fi
done

if [[ ! -f dependents.json ]]; then
  printf 'missing dependents.json\n' >&2
  exit 1
fi

if [[ ! -f original/pngtest.png ]]; then
  printf 'missing original/pngtest.png\n' >&2
  exit 1
fi

for pattern in libpng16-16t64_*.deb libpng-dev_*.deb libpng-tools_*.deb; do
  if ! compgen -G "$pattern" >/dev/null; then
    printf 'missing required package matching %s\n' "$pattern" >&2
    exit 1
  fi
done

expected_count="${#required_dependents[@]}"
actual_count="$(jq -r '.dependents | length' dependents.json)"
if [[ "$actual_count" != "$expected_count" ]]; then
  printf 'dependents.json count mismatch: expected %s, found %s\n' "$expected_count" "$actual_count" >&2
  exit 1
fi

for dependent in "${required_dependents[@]}"; do
  jq -e --arg name "$dependent" '.dependents[] | select(.name == $name)' dependents.json >/dev/null
done

docker build -t "$IMAGE_TAG" -f - . <<'DOCKERFILE'
FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      ca-certificates \
      dbus-x11 \
      feh \
      file \
      gcc \
      gimp \
      libcairo2-dev \
      libgdk-pixbuf-2.0-dev \
      libsdl2-dev \
      libsdl2-image-dev \
      libwebkit2gtk-4.1-dev \
      libreoffice-draw \
      make \
      netpbm \
      pkg-config \
      pngquant \
      python3-minimal \
      r-base \
      r-cran-png \
      scribus \
      x11-utils \
      xsane \
      xvfb \
 && rm -rf /var/lib/apt/lists/*

COPY libpng16-16t64_*.deb libpng-dev_*.deb libpng-tools_*.deb /tmp/local-libpng/

RUN dpkg -i /tmp/local-libpng/*.deb \
 && rm -rf /tmp/local-libpng

COPY original/pngtest.png /opt/fixtures/input.png

RUN useradd -m -s /bin/bash tester \
 && mkdir -p /home/tester/work \
 && chown -R tester:tester /home/tester

USER tester
WORKDIR /home/tester/work
DOCKERFILE

docker run --rm -i "$IMAGE_TAG" bash <<'EOF'
set -euo pipefail

log() {
  printf '==> %s\n' "$1"
}

require_nonempty_file() {
  local path="$1"
  if [[ ! -s "$path" ]]; then
    printf 'expected non-empty file: %s\n' "$path" >&2
    exit 1
  fi
}

cp /opt/fixtures/input.png ./input.png

log "GIMP"
timeout 120 gimp-console-2.10 -i -d -f \
  -b "(let* ((image (car (gimp-file-load RUN-NONINTERACTIVE \"$(pwd)/input.png\" \"$(pwd)/input.png\"))) (drawable (car (gimp-image-get-active-layer image)))) (file-png-save-defaults RUN-NONINTERACTIVE image drawable \"$(pwd)/gimp-out.png\" \"$(pwd)/gimp-out.png\") (gimp-image-delete image))" \
  -b "(gimp-quit 0)" \
  >/tmp/gimp.log 2>&1
require_nonempty_file "$(pwd)/gimp-out.png"
file "$(pwd)/gimp-out.png" | grep -F 'PNG image data' >/dev/null

log "LibreOffice Draw"
mkdir -p lo-profile lo-out
timeout 120 libreoffice --headless \
  "-env:UserInstallation=file://$(pwd)/lo-profile" \
  --convert-to pdf \
  --outdir "$(pwd)/lo-out" \
  "$(pwd)/input.png" \
  >/tmp/libreoffice.log 2>&1
require_nonempty_file "$(pwd)/lo-out/input.pdf"
file "$(pwd)/lo-out/input.pdf" | grep -F 'PDF document' >/dev/null

log "Scribus"
mkdir -p scribus
cat > scribus/test.py <<'PY'
import os
import scribus

out_dir = os.path.abspath("scribus")
image_path = os.path.abspath("input.png")
doc_path = os.path.join(out_dir, "doc.sla")
pdf_path = os.path.join(out_dir, "doc.pdf")

if not scribus.haveDoc():
    if not scribus.newDocument((300, 300), (10, 10, 10, 10), scribus.PORTRAIT, 1, scribus.UNIT_POINTS, scribus.NOFACINGPAGES, scribus.FIRSTPAGELEFT, 1):
        raise SystemExit("newDocument failed")

frame = scribus.createImage(20, 20, 200, 200)
scribus.loadImage(image_path, frame)
scribus.setScaleImageToFrame(True, True, frame)
scribus.saveDocAs(doc_path)
pdf = scribus.PDFfile()
pdf.file = pdf_path
pdf.pages = [1]
pdf.save()
scribus.closeDoc()
PY
timeout 120 xvfb-run -a scribus -g -ns -py "$(pwd)/scribus/test.py" >/tmp/scribus.log 2>&1
require_nonempty_file "$(pwd)/scribus/doc.sla"
require_nonempty_file "$(pwd)/scribus/doc.pdf"
file "$(pwd)/scribus/doc.pdf" | grep -F 'PDF document' >/dev/null

log "WebKitGTK"
mkdir -p webkit
cat > webkit/index.html <<'HTML'
<!doctype html>
<html><body><img id="png" src="file:///home/tester/work/input.png"></body></html>
HTML
cat > webkit/webkit.c <<'C'
#include <gtk/gtk.h>
#include <jsc/jsc.h>
#include <stdlib.h>
#include <webkit2/webkit2.h>

static void fail_and_quit(GMainLoop *loop, const char *message) {
    g_printerr("%s\n", message);
    g_main_loop_quit(loop);
    exit(1);
}

static void on_js_finished(GObject *object, GAsyncResult *result, gpointer user_data) {
    GMainLoop *loop = user_data;
    GError *error = NULL;
    JSCValue *value = webkit_web_view_evaluate_javascript_finish(WEBKIT_WEB_VIEW(object), result, &error);
    if (value == NULL) {
        fail_and_quit(loop, error ? error->message : "javascript evaluation failed");
    }

    char *text = jsc_value_to_string(value);
    gboolean ok = g_strcmp0(text, "ok") == 0;
    g_free(text);
    g_object_unref(value);

    if (!ok) {
        fail_and_quit(loop, "unexpected javascript result");
    }

    g_main_loop_quit(loop);
}

static void on_load_changed(WebKitWebView *web_view, WebKitLoadEvent load_event, gpointer user_data) {
    if (load_event == WEBKIT_LOAD_FINISHED) {
        webkit_web_view_evaluate_javascript(
            web_view,
            "(function(){const img=document.getElementById('png'); return (img && img.complete && img.naturalWidth > 0) ? 'ok' : 'fail';})()",
            -1,
            NULL,
            NULL,
            NULL,
            on_js_finished,
            user_data);
    }
}

int main(int argc, char **argv) {
    gtk_init(&argc, &argv);
    GtkWidget *window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    GtkWidget *view = webkit_web_view_new();
    gtk_container_add(GTK_CONTAINER(window), view);
    GMainLoop *loop = g_main_loop_new(NULL, FALSE);
    g_signal_connect(view, "load-changed", G_CALLBACK(on_load_changed), loop);
    webkit_web_view_load_uri(WEBKIT_WEB_VIEW(view), "file:///home/tester/work/webkit/index.html");
    gtk_widget_show_all(window);
    g_main_loop_run(loop);
    g_main_loop_unref(loop);
    return 0;
}
C
gcc webkit/webkit.c -o webkit/webkit-test $(pkg-config --cflags --libs webkit2gtk-4.1)
timeout 120 xvfb-run -a ./webkit/webkit-test >/tmp/webkit.log 2>&1

log "GDK Pixbuf"
mkdir -p gdk-pixbuf
cat > gdk-pixbuf/test.c <<'C'
#include <gdk-pixbuf/gdk-pixbuf.h>
#include <stdio.h>

int main(void) {
    GError *error = NULL;
    GdkPixbuf *pixbuf = gdk_pixbuf_new_from_file("/home/tester/work/input.png", &error);
    if (pixbuf == NULL) {
        fprintf(stderr, "%s\n", error ? error->message : "gdk-pixbuf load failed");
        return 1;
    }
    int width = gdk_pixbuf_get_width(pixbuf);
    int height = gdk_pixbuf_get_height(pixbuf);
    g_object_unref(pixbuf);
    return (width > 0 && height > 0) ? 0 : 1;
}
C
gcc gdk-pixbuf/test.c -o gdk-pixbuf/test $(pkg-config --cflags --libs gdk-pixbuf-2.0)
./gdk-pixbuf/test

log "Cairo"
mkdir -p cairo
cat > cairo/test.c <<'C'
#include <cairo.h>

int main(void) {
    cairo_surface_t *surface = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, 32, 32);
    cairo_t *cr = cairo_create(surface);
    cairo_set_source_rgb(cr, 0.1, 0.7, 0.2);
    cairo_paint(cr);
    cairo_destroy(cr);
    cairo_status_t status = cairo_surface_write_to_png(surface, "/home/tester/work/cairo/out.png");
    cairo_surface_destroy(surface);
    return status == CAIRO_STATUS_SUCCESS ? 0 : 1;
}
C
gcc cairo/test.c -o cairo/test $(pkg-config --cflags --libs cairo)
./cairo/test
require_nonempty_file "$(pwd)/cairo/out.png"
file "$(pwd)/cairo/out.png" | grep -F 'PNG image data' >/dev/null

log "SDL2_image"
mkdir -p sdl2-image
cat > sdl2-image/test.c <<'C'
#include <SDL.h>
#include <SDL_image.h>

int main(void) {
    if (SDL_Init(0) != 0) {
        return 1;
    }

    int flags = IMG_Init(IMG_INIT_PNG);
    if ((flags & IMG_INIT_PNG) == 0) {
        SDL_Quit();
        return 1;
    }

    SDL_Surface *surface = IMG_Load("/home/tester/work/input.png");
    if (surface == NULL) {
        IMG_Quit();
        SDL_Quit();
        return 1;
    }

    int ok = surface->w > 0 && surface->h > 0;
    SDL_FreeSurface(surface);
    IMG_Quit();
    SDL_Quit();
    return ok ? 0 : 1;
}
C
gcc sdl2-image/test.c -o sdl2-image/test $(pkg-config --cflags --libs SDL2_image)
./sdl2-image/test

log "feh"
feh --loadable "$(pwd)/input.png" >/tmp/feh.log 2>&1

log "Netpbm"
mkdir -p netpbm
pngtopnm "$(pwd)/input.png" > "$(pwd)/netpbm/out.ppm"
pnmtopng "$(pwd)/netpbm/out.ppm" > "$(pwd)/netpbm/roundtrip.png"
require_nonempty_file "$(pwd)/netpbm/roundtrip.png"

log "XSane"
timeout 150 xvfb-run -a bash -lc '
  xsane -s -N "'"$(pwd)"'/xsane-out" test:0 >/tmp/xsane.log 2>&1 &
  pid=$!
  sleep 20
  xwininfo -root -tree > "'"$(pwd)"'/xsane-tree.txt"
  kill "$pid" || true
  wait "$pid" || true
'
grep -F '"xsane 0.999' "$(pwd)/xsane-tree.txt" >/dev/null
grep -F '"Preview ' "$(pwd)/xsane-tree.txt" >/dev/null
grep -F '"Standard options ' "$(pwd)/xsane-tree.txt" >/dev/null

log "R png package"
mkdir -p r-png
Rscript -e 'library(png); img <- readPNG("input.png"); stopifnot(dim(img)[1] > 0, dim(img)[2] > 0); writePNG(img, "r-png/out.png")'
require_nonempty_file "$(pwd)/r-png/out.png"
file "$(pwd)/r-png/out.png" | grep -F 'PNG image data' >/dev/null

log "pngquant"
mkdir -p pngquant
pngquant --force --output "$(pwd)/pngquant/out.png" "$(pwd)/input.png" >/tmp/pngquant.log 2>&1
require_nonempty_file "$(pwd)/pngquant/out.png"
file "$(pwd)/pngquant/out.png" | grep -F 'PNG image data' >/dev/null

log "All dependent smoke tests passed"
EOF
