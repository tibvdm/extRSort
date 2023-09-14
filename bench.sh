#!/bin/bash

KB=1000
MB=$((1000 * KB))
GB=$((1000 * MB))

EXTRSORT_EXECUTABLE="./target/release/sorter"
GNU_SORT_EXECUTABLE="sort"

AMOUNT_OF_ITERATIONS=1

BUFFER_SIZE_SMALL="$((500 * $KB)) $((1 * $MB)) $((2 * $MB)) $((4 * $MB)) $((8 * $MB)) $((16 * $MB)) $((32 * $MB)) $((64 * $MB))"
THREADS_SMALL="1 2 4 8"

BUFFER_SIZE_MEDIUM="$((200 * $MB))"
THREADS_MEDIUM="1 2 4 8"

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

generate_medium_data_file() {
    if [[ -f "$1" ]]; then
        logn "File already exists"
    else
	python3 create_data_file.py --min-length 5 --max-length 150 --amount 50000000 --batch-size 100000 > "$1"
    fi
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
    timings=$(LC_ALL=C cat $1 | { time $EXTRSORT_EXECUTABLE --parallel "$2" --buffer-size "$3" > "/dev/null"; } 2>&1)

    real=$(echo $timings | cut -d' ' -f2)
    user=$(echo $timings | cut -d' ' -f4)
    sys=$(echo $timings | cut -d' ' -f6)

    echo "$2,$3,$real,$user,$sys"
}

time_gnu_sort_c() {
    timings=$(LC_ALL=C cat $1 | { time $GNU_SORT_EXECUTABLE --parallel="$2" --buffer-size="$3" --temporary-directory="/mnt/data/tmp" > "/dev/null"; } 2>&1)

    real=$(echo $timings | cut -d' ' -f2)
    user=$(echo $timings | cut -d' ' -f4)
    sys=$(echo $timings | cut -d' ' -f6)

    echo "$2,$3,$real,$user,$sys"
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
	thread_string=$(format_threads $threads)
        for buffer_size in $4; do
            for iteration in $(seq 1 $2); do
                log "\r\033[K\033[36m[ $current_configuration / 32 ]\033[m Benchmarking with $thread_string and buffer size $buffer_size ($iteration / $2)"
                time_gnu_sort_c $1 $threads $buffer_size
            done

            current_configuration=$(($current_configuration + 1))

            logn
        done
    done
}

bench_small() {
    logn "Generating small data set"
    generate_small_data_file "/mnt/data/tmp/small.unsorted"
    logn

    logn "\033[32mBenchmarking extrsort\033[0m"
    echo "threads,buffer_size,real,user,sys" > "data/extrsort.timings.small.csv"
#    bench_extsort "data/small.unsorted" "$AMOUNT_OF_ITERATIONS" "$THREADS_SMALL" "$BUFFER_SIZE_SMALL" >> "data/extrsort.timings.small.csv"
    logn

    logn "\033[32mBenchmarking gnu sort (C)\033[0m"
    echo "threads,buffer_size,real,user,sys" > "data/gnu_sort_c.timings.small.csv"
    bench_gnu_sort_c "data/small.unsorted" "$AMOUNT_OF_ITERATIONS" "$THREADS_SMALL" "$BUFFER_SIZE_SMALL" >> "data/gnu_sort_c.timings.small.csv"

    #rm "data/small.unsorted"

    logn
}

bench_medium() {
    logn "Generating medium data set"
    generate_medium_data_file "/mnt/data/tmp/medium.unsorted"
    logn

    logn "\033[32mBenchmarking extrsort\033[0m"
    echo "threads,buffer_size,real,user,sys" > "data/extrsort.timings.medium.csv"
    bench_extsort "/mnt/data/tmp/medium.unsorted" "$AMOUNT_OF_ITERATIONS" "$THREADS_MEDIUM" "$BUFFER_SIZE_MEDIUM" >> "data/extrsort.timings.medium.csv"
    logn

    logn "\033[32mBenchmarking gnu sort (C)\033[0m"
    echo "threads,buffer_size,real,user,sys" > "data/gnu_sort_c.timings.medium.csv"
    bench_gnu_sort_c "/mnt/data/tmp/medium.unsorted" "$AMOUNT_OF_ITERATIONS" "$THREADS_MEDIUM" "$BUFFER_SIZE_MEDIUM" >> "data/gnu_sort_c.timings.medium.csv"

    #rm "/mnt/data/tmp/medium.unsorted"

    logn
}

# ==================
# ====== Main ======
# ==================

# Build extrsort
build_extrsort

# Run benchmarks
bench_medium
