#![allow(dead_code)]
#![allow(unused_mut)]

use std::fs::File;
use std::io::Read;
use xxh3::hash128_with_seed;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Chunk {
    end: usize,
    start: usize,
    is_present: bool,
    problem_bytes: Vec<u8>,
}

impl Chunk {
    fn new(
        idx: usize, 
        bytes: Vec<u8>, 
        c_size: usize, 
        is_present: bool
    ) -> Self {
        Self {
            is_present,
            start: idx * c_size,
            problem_bytes: bytes,
            end: (idx * c_size) + c_size,
        }
    }
}

type HashList = Vec<u128>;
type DeltaMap = HashMap<usize, Chunk>;
type PositionMap<'a> = HashMap<&'a u128, usize>;

#[derive(Debug)]
pub struct Common { pub chunk_size: usize, }

impl Common {
    // function to find diff between the files
    pub fn diff(
        &self, 
        hash_list: &HashList, 
        path: &str
    ) -> Option<DeltaMap> {
        let file = File::open(path);

        if let Err(e) = file {
            println!("\nFile/Path {}: {}\n", path, std::io::Error::last_os_error());  
            return None;
        }
        
        let mut file = file.unwrap();
        
        let m_dat = file.metadata().ok()?;
        if !self.has_at_least_2_chunks(m_dat.len() as usize) {
            println!("\nFile {} must contain at least 2 chunks!!\n", path);
            return None;
        }
        // trick to read char by char is here! (u8 buf of size 1)
        let mut buffer = [0u8; 1];
        let mut delta_map = DeltaMap::new();
        let mut problem_bytes = Vec::<u8>::new();
        let mut data_buf = Vec::with_capacity(self.chunk_size);
        let mut position_map = self.hash_list_to_position_map(hash_list);
        
        // iterate over file char by char
        loop {
            let read_count = file.read(&mut buffer).ok()?;
            // when there is nothing to read
            if read_count != 1 { break; }
            // push each char into buffer
            data_buf.push(buffer[0]);
            
            // continue until data_buf is of chunk_size 
            if(data_buf.len() < self.chunk_size) { continue; }

            // if data_buf didn't got cleared on previous iteration
            // because there, was no match then this condition 
            // will become true!
            if data_buf.len() == self.chunk_size + 1 {
                // push the first byte into problem_bytes
                problem_bytes.push(data_buf[0]);
                // skip first byte from buffer to become equal to chunk_size
                data_buf = data_buf.into_iter().skip(1).collect::<Vec<u8>>();
            }

            // try get position of chunk by hash
            let res = position_map.get(
                &hash128_with_seed(
                    &data_buf[..], 
                    0u64
                )
            );
            // if there is position, means this chunk is not mis-match
            if let Some(p) = res {
                let pos = p.clone();
                delta_map.insert(
                    pos, 
                    Chunk::new(
                        pos.clone(),
                        // save also problem_bytes related to this chunk
                        // collected by previous iterations
                        problem_bytes.clone(),
                        self.chunk_size, 
                        true
                    )
                );
                // clear buffers to start anew
                data_buf.drain(..);
                problem_bytes.drain(..);
            }
        }
        self.try_fill_missing_chunks(
            &hash_list, 
            &mut delta_map
        );
        Some(delta_map)
    }

    // function to fill chunks we missed during diff operation
    pub fn try_fill_missing_chunks(
        &self, 
        hash_list: &HashList, 
        deltas: &mut DeltaMap
    ) {
        for i in 0..hash_list.len() {
            if deltas.get(&i).is_none() {
                deltas.insert(
                    i,
                    Chunk::new(
                        i, 
                        Vec::new(), 
                        self.chunk_size, 
                        false
                    )
                );
            }
        }
    }
    
    pub fn hash_list_to_position_map<'a>(
        &'a self, 
        hash_list: &'a HashList
    ) -> PositionMap {
        let mut position_map: HashMap<&u128, usize> = HashMap::new();
        for (i, hash) in hash_list.into_iter().enumerate() {
            position_map.insert(&hash, i);
        }
        position_map
    }

    pub fn file_to_chunk_hash_list<'a>(
        &self, 
        path: &'a str
    ) -> Option<Vec<u128>> {
        let mut file = File::open(path);

        if let Err(e) = file {
            println!("\nFile/Path {}: {}\n", path, std::io::Error::last_os_error());  
            return None;
        }

        let mut file = file.unwrap();
        
        let m_dat = file.metadata().ok()?;

        if !self.has_at_least_2_chunks(m_dat.len() as usize) {
            println!("\nFile {} must contain at least 2 chunks!!\n", path);
            return None;
        }

        let mut buffer = [0u8; 1];
        let mut hash_list: Vec<u128> = Vec::new();
        let mut data_buf = Vec::with_capacity(self.chunk_size);
        loop {
            let read_count = file.read(&mut buffer).ok()?;
            
            if read_count != 1 {
                hash_list.push(hash128_with_seed(&data_buf[..], 0u64));
                break;
            }

            data_buf.push(buffer[0]);
            // println!("len: {}", data_buf.len());
            if(data_buf.len() == self.chunk_size) {
                hash_list.push(hash128_with_seed(&data_buf[..], 0u64));
                data_buf.drain(..);
            }
        }
        Some(hash_list)
    }

    fn has_at_least_2_chunks(&self, f_size: usize) -> bool {
        self.chunk_size <= f_size - 1 
    }
}








