# -*-Makefile-*-

build:
    cargo build

build-release:
    cargo build --release

test colours='':
     cargo {{colours}} nextest run

clean:
    cargo clean

debug bin *args:
    cargo run           --bin {{bin}} -- {{args}}

run bin *args:
    cargo run --release --bin {{bin}} -- {{args}}

hdfy folder *args: build-release
  just run hdfy -i {{folder}} -o {{folder}}.h5 {{args}}

hdfy-many folder *args: build-release
  #!/usr/bin/env sh

  njobs=$(find "{{folder}}" -mindepth 1 -maxdepth 1 -type d | wc -l)
  i=0

  for f in "{{folder}}"/*; do
      [ -d "$f" ] || continue
      i=$((i + 1))

      stdbuf -oL       ./target/release/hdfy -i "$f" -o "$f.h5" {{args}} --batch &
      echo "Process for folder $f has PID $!"
      if [ $((i % 11)) -eq 0 ]; then
          echo "Waiting to schedule more jobs ($((njobs-i)) remaining)"
          wait
      fi
  done
  echo "Waiting for last few jobs to finish"
  wait
