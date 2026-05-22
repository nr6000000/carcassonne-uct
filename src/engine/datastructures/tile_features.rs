use std::hash::Hash;
use std::fmt::Debug;

use heapless::index_map::FnvIndexMap;
use strum::EnumCount;

use crate::engine::datastructures::direction::Direction;
use crate::engine::datastructures::rel_direction::RelDirection;
use crate::engine::tile::{Feature, Rotation};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct TileFeatures {
    features: [Feature;4]
}

impl Debug for TileFeatures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn feature_to_ch(feature: &Feature) -> char {
            match feature {
                Feature::City => 'C',
                Feature::Road => 'R',
                Feature::Field => 'F',
            }
        }

        write!(
            f, 
            "{}{}{}{}", 
            feature_to_ch(&self.features[0]),
            feature_to_ch(&self.features[1]),
            feature_to_ch(&self.features[2]),
            feature_to_ch(&self.features[3]),
        )?;
        Ok(())
    }
}

fn order_value(features: &&[Feature; 4]) -> i32 {
    let base = Feature::COUNT as i32;

    base.pow(3) * features[0] as i32 +
    base.pow(2) * features[1] as i32 + 
    base * features[2] as i32 +
    1 * features[3] as i32 
}

impl TileFeatures {
    pub fn new(n: Feature, e: Feature, s: Feature, w: Feature) -> Self {
        let orders = [
            [n, e, s, w],
            [e, s, w, n],
            [s, w, n, e],
            [w, n, e, s],
        ];

        let canonical_order= orders
            .iter()
            .min_by_key(order_value)
            .unwrap();

        Self { features: *canonical_order }
    }

    pub fn get(&self) -> &[Feature] {
        &self.features
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Feature> {
        self.features.iter()
    }

    pub fn get_feature(&self, dir: &RelDirection) -> &Feature {
        match *dir {
            RelDirection::Up => &self.features[0],
            RelDirection::Right => &self.features[1],
            RelDirection::Down => &self.features[2],
            RelDirection::Left => &self.features[3],
            _ => panic!("Unexpected RelDirection"),
        }
    }

    pub fn fix_rotation(&self, rot: &Rotation) -> FnvIndexMap<Direction, Feature, 4>{
        self.features.iter().enumerate()
            .map(|(i, f)| {
                (
                    RelDirection::from_index((i*2) as u8).to_abs(&rot), 
                    *f
                )
            })
            .collect()
    }
}

impl FromIterator<Feature> for TileFeatures {
    fn from_iter<T: IntoIterator<Item = Feature>>(iter: T) -> Self {
        let mut into_iter = iter.into_iter();
        Self::new(
            into_iter.next().unwrap(),
            into_iter.next().unwrap(),
            into_iter.next().unwrap(),
            into_iter.next().unwrap(),
        )
    }
}