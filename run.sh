FILE=$1

cargo build --release

./tools/target/release/tester ./target/release/ahc018 < tools/in/$FILE.txt > tools/out/$FILE.txt

pbcopy < tools/out/$FILE.txt

python3 visualizer.py $1
