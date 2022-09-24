## **rdiff (rust-diff)** 
### *Rolling Hash based file diff tool coded in rust-lang*
***
***
#### Requirements:
    - rustc 1.63.0 stable
    - linux or macos preferably, may also run on windows

#### Compilation:
    ```
    make
    ```
if no make utility:
    ```
    cargo build --release
    ```

#### Usage:
    ```
    ./target/release/rdiff file1 file2 <chunk_size>
    ```

##### Default chunk size is 4 if none provided

#### Examples:
    ```
        ./target/release/rdiff file1 file2
        ./target/release/rdiff file1 file2 5
        ./target/release/rdiff file1 file2 16
        ./target/release/rdiff file1 file2 33
    ```
***
***
##### MIT License