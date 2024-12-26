use std::collections::HashMap;

use osmio::{Lat, Lon, Node, Way};
use osmio::{OSMObjBase, ObjId};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use smol_str::SmolStr;

impl
    From<(
        osmio::obj_types::StringWay,
        &mut HashMap<ObjId, HighwayNode>,
    )> for Highway
{
    fn from(
        (value, highway_nodes): (
            osmio::obj_types::StringWay,
            &mut HashMap<ObjId, HighwayNode>,
        ),
    ) -> Self {
        let tags = value
            .tags()
            .map(|(a, b)| (SmolStr::new(a), SmolStr::new(b)))
            .collect();
        let nodes = value
            .nodes()
            .iter()
            .map(|node_id| highway_nodes.get(node_id).unwrap().clone())
            .collect();
        Self {
            id: value.id(),
            tags,
            nodes,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Highway {
    pub id: ObjId,
    pub tags: SmallVec<[(SmolStr, SmolStr); 6]>,
    pub nodes: SmallVec<[HighwayNode; 10]>,
}

impl From<osmio::obj_types::StringNode> for HighwayNode {
    fn from(value: osmio::obj_types::StringNode) -> Self {
        let tags = value
            .tags()
            .map(|(a, b)| (SmolStr::new(a), SmolStr::new(b)))
            .collect();
        let (latitude, longitude) = value.lat_lon().unwrap();
        Self {
            id: value.id(),
            tags,
            latitude,
            longitude,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighwayNode {
    pub id: ObjId,
    pub tags: SmallVec<[(SmolStr, SmolStr); 1]>,
    pub latitude: Lat,
    pub longitude: Lon,
}

impl PartialEq for HighwayNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
