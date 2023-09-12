#!/bin/bash

KB=1000
MB=$((1000 * KB))
GB=$((1000 * MB))

EXTRSORT_EXECUTABLE="./target/release/sorter"
GNU_SORT_EXECUTABLE="gsort"

AMOUNT_OF_ITERATIONS=10

BUFFER_SIZE_SMALL="$((500 * $KB)) $((1 * $MB)) $((2 * $MB)) $((4 * $MB)) $((8 * $MB)) $((16 * $MB)) $((32 * $MB)) $((64 * $MB))"
THREADS_SMALL="1 2 4 8"

log() {
    echo -en "$1" 1>&2
}

logn() {
    echo -e "$1" 1>&2
}

build_extrsort() {
    cargo build --release --quiet
}

# =============================
# ====== Data generation ======
# =============================

generate_small_data_file() {
    python3 create_data_file.py --min-length 5 --max-length 150 --amount 2500000 --batch-size 10000 > "$1"
}

# ==================================
# ====== Formatting functions ======
# ==================================

format_threads() {
    [[ $1 -eq 1 ]] && echo "1 thread" || echo "$1 threads"
}

format_buffer_size() {
    buffer_size=$1
    division_counter=0
    while [[ $buffer_size -ge 1000 ]]; do
        buffer_size=$(($buffer_size / 1000))
        division_counter=$(($division_counter + 1))
    done

    suffixes=("B" "KB" "MB" "GB" "TB")
    echo "$buffer_size ${suffixes[$division_counter]}"
}

# ==============================
# ====== Timing functions ======
# ==============================

time_extrsort() {
    timings=$(cat $1 | { time $EXTRSORT_EXECUTABLE --parallel "$2" --buffer-size "$3" > "/dev/null"; } 2>&1)
    
    real=$(echo $timings | cut -d' ' -f2)
    user=$(echo $timings | cut -d' ' -f4)
    sys=$(echo $timings | cut -d' ' -f6)

    echo "$2,$3,$real,$user,$sys" >> "data/extrsort.timings.small.csv"
}

time_gnu_sort_c() {
    timings=$(LC=ALL cat $1 | { time $GNU_SORT_EXECUTABLE --parallel="$2" --buffer-size="$3" > "/dev/null"; } 2>&1)
    
    real=$(echo $timings | cut -d' ' -f2)
    user=$(echo $timings | cut -d' ' -f4)
    sys=$(echo $timings | cut -d' ' -f6)

    echo "$2,$3,$real,$user,$sys" >> "data/gnu_sort_c.timings.small.csv"
}

# =============================
# ====== Bench functions ======
# =============================

bench_extsort() {
    current_configuration=1
    for threads in $3; do
        thread_string=$(format_threads $threads)
        for buffer_size in $4; do
            buffer_size_string=$(format_buffer_size $buffer_size)
            for iteration in $(seq 1 $2); do
                log "\r\033[K\033[36m[ $current_configuration / 32 ]\033[m Benchmarking with $thread_string and buffer size $buffer_size_string ($iteration / $2)"
                time_extrsort $1 $threads $buffer_size
            done

            current_configuration=$(($current_configuration + 1))

            logn
        done
    done
}

bench_gnu_sort_c() {
    current_configuration=1
    for threads in $3; do
        for buffer_size in $4; do
            for iteration in $(seq 1 $2); do
                log "\r\033[K\033[36m[ $current_configuration / 32 ]\033[m Benchmarking with $threads threads and buffer size $buffer_size ($iteration / $2)"
                time_gnu_sort_c $1 $threads $buffer_size
            done

            current_configuration=$(($current_configuration + 1))

            logn
        done
    done
}

bench_small() {
    logn "Generating small data set"
    generate_small_data_file "data/small.unsorted"
    logn

    # logn "\033[32mBenchmarking extrsort\033[0m"
    # echo "threads,buffer_size,real,user,sys" > "data/extrsort.timings.small.csv"
    # bench_extsort "data/small.unsorted" "$AMOUNT_OF_ITERATIONS" "$THREADS_SMALL" "$BUFFER_SIZE_SMALL"
    # logn

    logn "\033[32mBenchmarking gnu sort (C)\033[0m"
    echo "threads,buffer_size,real,user,sys" > "data/gnu_sort_c.timings.small.csv"
    bench_gnu_sort_c "data/small.unsorted" "$AMOUNT_OF_ITERATIONS" "$THREADS_SMALL" "$BUFFER_SIZE_SMALL"

    # rm "data/small.unsorted"

    logn
}

# ==================
# ====== Main ======
# ==================

# Build extrsort
build_extrsort

# Run benchmarks
bench_small
