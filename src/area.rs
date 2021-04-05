use crate::varint;
use crate::{label, point, tags};
use desert::ToBytesLE;
use earcutr;
use failure::Error;

#[test]
fn peer_area() -> Result<(),Error> {
    let tags = vec![
        ("source", "bing"),
        ("boundary", "protected_area"),
        ("tiger:cfcc", "A41"),
    ];
    let positions: Vec<f32> = vec![
        31.184799400000003, 29.897739500000004,
        31.184888100000002, 29.898801400000004,
        31.184858400000003, 29.8983899,
    ];
    let id: u64 = 234941233;
    let mut area = PeerArea::from_tags(id, &tags)?;
    area.push(&positions, &vec![]);

    let bytes = area.to_bytes_le().unwrap();
    assert_eq!(
        "03c901b1d6837003787af941922eef41a77af941bf30ef41977af941e72fef410101000200",
        hex::encode(bytes)
    );
    Ok(())
}

#[derive(Debug)]
pub struct PeerArea {
    pub id: u64,
    pub feature_type: u64,
    pub labels: Vec<u8>,
    pub positions: Vec<f32>,
    pub cells: Vec<usize>,
}

impl PeerArea {
    pub fn from_tags(id: u64, tags: &[(&str, &str)]) -> Result<PeerArea,Error> {
        let (feature_type, labels) = tags::parse(tags)?;
        Ok(Self { id, feature_type, labels, positions: vec![], cells: vec![] })
    }
    pub fn new(id: u64, feature_type: u64, labels: &[u8]) -> PeerArea {
        Self {
            id,
            feature_type,
            labels: labels.to_vec(),
            positions: vec![],
            cells: vec![]
        }
    }
    pub fn push(&mut self, positions: &[f32], holes: &[usize]) -> () {
        let cells = earcutr::earcut(
            &positions.iter().map(|p| *p as f64).collect(),
            &holes.to_vec(),
            2
        );
        let offset = self.positions.len() / 2;
        self.cells.extend(cells.iter().map(|c| c+offset).collect::<Vec<usize>>());
        self.positions.extend_from_slice(positions);
    }
}

impl ToBytesLE for PeerArea {
    fn to_bytes_le(&self) -> Result<Vec<u8>, Error> {
        let pcount = self.positions.len()/2;
        let ft_length = varint::length(self.feature_type);
        let id_length = varint::length(self.id);
        let pcount_length = varint::length(pcount as u64);
        let clen = varint::length((self.cells.len() / 3) as u64);
        let clen_data = self.cells.iter()
            .fold(0, |acc, c| acc + varint::length(*c as u64));

        let mut buf = vec![
            0u8;
            1 + ft_length
                + id_length
                + pcount_length
                + (2 * 4 * pcount)
                + clen
                + clen_data
                + self.labels.len()
        ];

        let mut offset = 0;
        buf[offset] = 0x03;

        offset += 1;
        offset += varint::encode_with_offset(self.feature_type, &mut buf, offset)?;
        offset += varint::encode_with_offset(self.id, &mut buf, offset)?;
        offset += varint::encode_with_offset(pcount as u64, &mut buf, offset)?;

        // positions
        for p in self.positions.iter() {
            offset += point::encode_with_offset(*p, &mut buf, offset)?;
        }

        offset += varint::encode_with_offset((self.cells.len()/3) as u64, &mut buf, offset)?;

        // cells
        for &cell in self.cells.iter() {
            offset += varint::encode_with_offset(cell as u64, &mut buf, offset)?;
        }

        label::encode_with_offset(&self.labels, &mut buf, offset);
        return Ok(buf);
    }
}
