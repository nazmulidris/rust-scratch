## Summary of memory latency

**Key Takeaway:** The CPU's cache hierarchy (L1, L2, and often L3) acts as a crucial
intermediary, bridging the massive performance gap between the incredibly fast CPU
registers and the much slower main memory and storage. Cache lines are the fundamental
building blocks that enable this efficient data movement and significantly impact overall
system performance.

**In summary, the relative latencies look something like this (very approximate):** Here's
a table summarizing the relative latencies:

| Memory Type | Relative Latency (vs. Register) |
| ----------- | ------------------------------- |
| Register    | 1x                              |
| L1 Cache    | 10x - 50x                       |
| L2 Cache    | 50x - 200x                      |
| RAM         | 500x - 1000x                    |
| SSD         | 10,000x - 100,000x              |

## Summary of Cache Lines

A **cache line** is the fundamental unit of data transfer between the CPU's cache
hierarchy and the main memory. On the 14th gen Intel CPUs it is 64 bytes, its primary
function is to enhance performance by fetching and storing data in larger blocks, thereby
reducing the frequency of slower main memory accesses. It is 128 bytes or twice as large
on an Apple M4 chip.

**How they work:** When the CPU requires data not present in the cache (a **cache miss**),
an entire cache line containing that data is retrieved from main memory. Similarly, when
data is written, the corresponding cache line is updated and eventually written back to
main memory.

**Impact on Memory Latency:** Cache lines significantly influence memory latency:

- **Spatial Locality:** By fetching a block of contiguous data, cache lines exploit the
  tendency of programs to access nearby memory locations, minimizing subsequent memory
  accesses.
- **Cache Miss Penalty:** While a cache miss incurs a substantial latency penalty to fetch
  the entire line, this is often offset by the fact that a larger chunk of potentially
  needed data is brought into the cache at once.
- **Bandwidth Utilization:** Transferring data in larger cache line units optimizes the
  use of the available memory bandwidth compared to numerous small transfers.

**Example Benefit:** Accessing elements of an array sequentially demonstrates the
advantage. With a 64-byte cache line and 4-byte integers, fetching one integer brings 15
neighboring integers into the cache, likely satisfying future access requests without
needing to go back to main memory.

## Order of Magnitude Latency Differences (Intel 14th Gen, DDR4-5200, PCIe 4 SSD)

It's important to understand that these are _relative_ order-of-magnitude estimates and
can vary based on specific workloads, system configurations, and the exact characteristics
of the components. However, they provide a good sense of the performance hierarchy:

1.  **Fetching Data from a Register:** This is the fastest operation, happening within the
    CPU core itself.

    - **Latency:** Sub-nanosecond (on the order of picoseconds). Let's consider this as
      **~1 unit of time**.

2.  **Fetching Data from L1 Cache:** The L1 cache is the smallest and closest cache to the
    CPU cores.

    - **Latency:** Around 1-5 nanoseconds. This is still incredibly fast, but noticeably
      slower than register access.
    - **Relative Order of Magnitude:** **~10x - 50x** slower than register access.

3.  **Fetching Data from L2 Cache:** The L2 cache is larger and slightly further away than
    L1.

    - **Latency:** Around 5-20 nanoseconds.
    - **Relative Order of Magnitude:** **~50x - 200x** slower than register access.

4.  **Fetching Data from Main Memory (DDR4-5200):** Accessing RAM is significantly slower
    than cache access. DDR4-5200 specifies the data transfer rate, but the actual latency
    to fetch data involves factors like CAS latency and command cycles.

    - **Latency:** Around 50-100 nanoseconds (or even higher depending on the specific
      timings and system load).
    - **Relative Order of Magnitude:** **~500x - 1000x** slower than register access.

5.  **Fetching Data from SSD (PCIe 4):** Accessing an SSD is orders of magnitude slower
    than RAM, although much faster than traditional hard drives. PCIe 4 offers high
    bandwidth, but the latency for a random access is still considerable in CPU time
    scales.
    - **Latency:** Tens to hundreds of _microseconds_ (thousands to hundreds of thousands
      of nanoseconds). Let's say around 10-100 microseconds for a typical random read.
    - **Relative Order of Magnitude:** **~10,000x - 100,000x** slower than register
      access.
