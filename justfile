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

hdfy-many folder *args: build-release
  #!/usr/bin/env sh

  for f in `find {{folder}}/* -type d`; do
      stdbuf -oL       ./target/release/hdfy -i $f -o $f.h5 {{args}} --batch &
      echo "JOB $i has PID $!"
      if [ $((i % 11)) -eq 10 ]; then
          echo "Waiting to schedule more jobs ($((njobs-i)) remaining)"
          wait
      fi
  done
  echo "Waiting for last few jobs to finish"
  wait
