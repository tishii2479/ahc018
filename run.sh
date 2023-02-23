FILE=$1

rm log/grid_*.txt
rm log/state_*.txt
rm log/vis_*.png

cargo build --features local --release

./tools/target/release/tester ./target/release/ahc018 < tools/in/$FILE.txt > tools/out/$FILE.txt

pbcopy < tools/out/$FILE.txt

python3 visualizer.py $1

open log/vis_movie.gif