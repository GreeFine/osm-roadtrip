use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use osmio::{OSMObjBase, OSMReader, ObjId};
use rayon::prelude::*;
use tracing::info;

use crate::models::{Highway, HighwayNode};

pub fn highway_cached<P: AsRef<Path>>(filepath: P) -> Result<Vec<Highway>> {
    let pb_reading_style = ProgressStyle::with_template(
        "[{elapsed_precise}] {percent:>3}% done. eta {eta:>4} {bar:10.cyan/blue} {bytes:>7}/{total_bytes:7} {per_sec:>12} {msg}\n",
        ).unwrap();

    let cache_filepath = filepath.as_ref().to_path_buf().with_file_name(format!(
        "_cache.highway.{}",
        filepath.as_ref().file_name().unwrap().to_str().unwrap()
    ));
    if PathBuf::from(&cache_filepath).exists() {
        info!("Cache found for highway, reading data.");

        let cache_fp = std::fs::File::open(&cache_filepath)?;
        let expected_len = cache_fp.metadata()?.len();
        let progress_bar = ProgressBar::new(cache_fp.metadata()?.len())
            .with_message(format!("Reading cache in {}", cache_filepath.display()))
            .with_style(pb_reading_style);
        let mut rdr = progress_bar.wrap_read(cache_fp);
        let mut buffer = Vec::with_capacity(expected_len.try_into()?);
        let _ = rdr.read_to_end(&mut buffer)?;
        let highways: Vec<Highway> = bincode::deserialize(&buffer)?;
        info!("Got {} highway from cache.", highways.len());

        return Ok(highways);
    }
    info!("Cache not found for highway, generating ...");

    let highway: Vec<Highway> = {
        let input_fp = std::fs::File::open(&filepath)?;
        let progress_bar = ProgressBar::new(input_fp.metadata()?.len())
            .with_message("Reading input file")
            .with_style(pb_reading_style);
        let rdr = progress_bar.wrap_read(input_fp);
        let mut reader = osmio::stringpbf::PBFReader::new(rdr);
        reader
            .ways()
            .filter(|way| way.has_tag("highway"))
            .map(From::from)
            .collect()
    };

    info!("Number of highway in file : {}", highway.len());
    let output_fp = std::fs::File::create_new(&cache_filepath)?;
    let encoded: Vec<u8> = bincode::serialize(&highway)?;
    let progress_bar = ProgressBar::new(encoded.len().try_into()?)
        .with_message(format!("Writing cache in {}", cache_filepath.display()));
    let mut wrt = progress_bar.wrap_write(output_fp);
    wrt.write_all(&encoded)?;

    Ok(highway)
}

pub fn nodes_cached<P: AsRef<Path>>(
    filepath: P,
    mut nodes_id: Vec<ObjId>,
) -> Result<Vec<HighwayNode>> {
    let pb_reading_style = ProgressStyle::with_template(
        "[{elapsed_precise}] {percent:>3}% done. eta {eta:>4} {bar:10.cyan/blue} {bytes:>7}/{total_bytes:7} {per_sec:>5} {msg}\n",
        ).unwrap();
    let pb_filtering_style = ProgressStyle::with_template(
        "[{elapsed_precise}] {pos}/{len}. eta {eta:>4} {bar:10.cyan/blue} {per_sec:>5} {msg}\n",
    )
    .unwrap();

    let cache_filepath = filepath.as_ref().to_path_buf().with_file_name(format!(
        "_cache.highway-nodes.{}",
        filepath.as_ref().file_name().unwrap().to_str().unwrap()
    ));
    if PathBuf::from(&cache_filepath).exists() {
        info!("Cache found for highway nodes, reading data.");

        let cache_fp = std::fs::File::open(&cache_filepath)?;
        let expected_len = cache_fp.metadata()?.len();
        let progress_bar = ProgressBar::new(cache_fp.metadata()?.len())
            .with_message(format!("Reading cache in {}", cache_filepath.display()))
            .with_style(pb_reading_style);
        let mut rdr = progress_bar.wrap_read(cache_fp);
        let mut buffer = Vec::with_capacity(expected_len.try_into()?);
        let _ = rdr.read_to_end(&mut buffer)?;
        let highways: Vec<HighwayNode> = bincode::deserialize(&buffer)?;
        info!("Got {} highway nodes from cache.", highways.len());

        return Ok(highways);
    }

    info!("Cache not found for highway nodes, generating ...");
    let highway_nodes = {
        info!("Extracting nodes from file");

        let mut nodes: Vec<HighwayNode> = {
            let input_fp = std::fs::File::open(&filepath)?;
            let progress_bar = ProgressBar::new(input_fp.metadata()?.len())
                .with_message("Reading input file")
                .with_style(pb_reading_style);
            let rdr = progress_bar.wrap_read(input_fp);
            let mut reader = osmio::stringpbf::PBFReader::new(rdr);
            reader.nodes().map(From::from).collect()
        };

        // Faster contains search
        nodes_id.sort();
        nodes_id.dedup();
        nodes.sort_by_key(|h| h.id);
        info!("Filtering to only highway nodes");

        let highway_nodes: Vec<_> = nodes
            .into_par_iter()
            .progress_with_style(pb_filtering_style)
            .filter(|node| nodes_id.binary_search(&node.id).is_ok())
            .collect();

        highway_nodes
    };

    info!("Number of highway nodes in file : {}", highway_nodes.len());
    let output_fp = std::fs::File::create_new(&cache_filepath)?;
    let encoded: Vec<u8> = bincode::serialize(&highway_nodes)?;
    let progress_bar = ProgressBar::new(encoded.len().try_into()?)
        .with_message(format!("Writing cache in {}", cache_filepath.display()));
    let mut wrt = progress_bar.wrap_write(output_fp);
    wrt.write_all(&encoded)?;

    Ok(highway_nodes)
}
