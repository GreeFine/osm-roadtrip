use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use osmio::{Node, OSMObjBase, OSMReader, ObjId};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tracing::info;

use crate::models::{Highway, HighwayNode};

pub fn highways<P: AsRef<Path>>(filepath: P) -> Result<Vec<Highway>> {
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

    let mut nodes = nodes(&filepath)?;
    let highway: Vec<Highway> = {
        let input_fp = std::fs::File::open(&filepath)?;
        let progress_bar = ProgressBar::new(input_fp.metadata()?.len())
            .with_message("Reading input file")
            .with_style(pb_reading_style);
        let rdr = progress_bar.wrap_read(input_fp);
        let mut reader = osmio::stringpbf::PBFReader::new(rdr);
        let mut result = Vec::new();
        for way in reader.ways().filter(|way| way.has_tag("highway")) {
            result.push(Highway::from((way, &mut nodes)));
        }
        result
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

fn nodes<P: AsRef<Path>>(filepath: P) -> Result<HashMap<ObjId, HighwayNode>> {
    let pb_reading_style = ProgressStyle::with_template(
        "[{elapsed_precise}] {percent:>3}% done. eta {eta:>4} {bar:10.cyan/blue} {bytes:>7}/{total_bytes:7} {per_sec:>5} {msg}\n",
        ).unwrap();

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
        let highways: HashMap<ObjId, HighwayNode> = bincode::deserialize(&buffer)?;
        info!("Got {} highway nodes from cache.", highways.len());

        return Ok(highways);
    }

    info!("Cache not found for highway nodes, generating ...");
    let highway_nodes: HashMap<ObjId, HighwayNode> = {
        info!("Extracting nodes from file");

        let input_fp = std::fs::File::open(&filepath)?;
        let progress_bar = ProgressBar::new(input_fp.metadata()?.len())
            .with_message("Reading input file")
            .with_style(pb_reading_style);
        let rdr = progress_bar.wrap_read(input_fp);
        let mut reader = osmio::stringpbf::PBFReader::new(rdr);
        let vec: Vec<HighwayNode> = reader
            .nodes()
            .filter(|n| n.has_lat_lon() && !n.deleted())
            .map(From::from)
            .collect();
        let mut result = HashMap::new();
        for item in vec {
            result.insert(item.id, item);
        }
        result
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

pub fn highway_connections(all_highways: &Vec<Highway>) -> HashMap<ObjId, Vec<ObjId>> {
    let pb_reading_style = ProgressStyle::with_template(
        "[{elapsed}] {percent:>3}% done. eta {eta:>4} {bar:10.cyan/blue} {per_sec:>5} {msg}\n",
    )
    .unwrap();

    all_highways[..10000]
        .par_iter()
        .progress_with_style(pb_reading_style)
        .map(|highway| {
            let ids_connecting = all_highways[..10000]
                .iter()
                .filter(|other| {
                    other.nodes.first() == highway.nodes.first()
                        || other.nodes.first() == highway.nodes.last()
                        || other.nodes.last() == highway.nodes.first()
                        || other.nodes.last() == highway.nodes.last()
                })
                .map(|other| other.id)
                .collect();
            (highway.id, ids_connecting)
        })
        .collect()
}
