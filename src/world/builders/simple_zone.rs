use bevy::log::info;

use crate::{common::{astar, AStarSettings, Distance, Grid, Rand}, projection::ZONE_SIZE, world::Terrain};

use super::{edge_snapshot, terrain_snapshot, ZoneBuilder, ZoneConstraints, ZoneData, ZoneSnapshot, ENABLE_ZONE_SNAPSHOTS};

#[derive(Default)]
pub struct SimpleZoneBuilder {
    snapshots: Vec<ZoneSnapshot>,
}

impl ZoneBuilder for SimpleZoneBuilder {
    fn build(&mut self, constraints: ZoneConstraints) -> ZoneData {
        let idx = constraints.idx;
        let mut r = Rand::seed(idx as u64);
        let terrains = vec![Terrain::Grass];
        let mut terrain = Grid::init_fill(ZONE_SIZE.0, ZONE_SIZE.1, |_, _| r.pick(&terrains));

        let mut rivers = vec![];
        let mut footpaths = vec![];

        if ENABLE_ZONE_SNAPSHOTS {
            self.snapshots.push(edge_snapshot(&constraints));
        }

        for (x, s) in constraints.south.iter().enumerate() {
            if *s == 1 {
                rivers.push((x, 0));
            }
            if *s == 2 {
                footpaths.push((x, 0));
            }
        }

        for (x, n) in constraints.north.iter().enumerate() {
            if *n == 1 {
                rivers.push((x, ZONE_SIZE.1 - 1));
            }
            if *n == 2 {
                footpaths.push((x, ZONE_SIZE.1 - 1));
            }
        }

        for (y, w) in constraints.west.iter().enumerate() {
            if *w == 1 {
                rivers.push((0, y));
            }
            if *w == 2 {
                footpaths.push((0, y));
            }
        }

        for (y, e) in constraints.east.iter().enumerate() {
            if *e == 1 {
                rivers.push((ZONE_SIZE.0 - 1, y));
            }
            if *e == 2 {
                footpaths.push((ZONE_SIZE.0 - 1, y));
            }
        }

        if footpaths.len() == 1 {
            footpaths.push((ZONE_SIZE.0 / 2, ZONE_SIZE.1 / 2));
        }

        if rivers.len() == 1 {
            rivers.push((ZONE_SIZE.0 / 2, ZONE_SIZE.1 / 2));
        }

        // every river should attempt to connect to every other river
        for (p1_idx, p1) in rivers.iter().enumerate() {
            (p1_idx..rivers.len()).for_each(|p2_idx| {
                let p2 = rivers[p2_idx];

                if p1_idx == p2_idx {
                    return;
                }

                info!("start {},{}", p1.0, p1.1);
                info!("goal {},{}", p2.0, p2.1);

                // find path between p1/p2, and set terrain!
                let result = astar(AStarSettings {
                    start: [
                        p1.0,
                        p1.1,
                        0,
                    ],
                    is_goal: |p| {
                        p[0] == p2.0 && p[1] == p2.1
                    },
                    cost: |a, b| {
                        let dist = Distance::diagonal([a[0] as i32, a[1] as i32, a[2] as i32], [b[0] as i32, b[1] as i32, b[2] as i32]);
                        let t = terrain.get(b[0], b[1]).unwrap();

                        dist * match t {
                            Terrain::Grass => 15.,
                            Terrain::Dirt => 15.,
                            Terrain::River => 1.,
                            Terrain::Footpath => 10.,
                        }
                    },
                    heuristic: |v| {
                        Distance::diagonal([v[0] as i32, v[1] as i32, v[2] as i32], [p2.0 as i32, p2.1 as i32, 0])
                    },
                    neighbors: |v| {
                        let mut n = vec![];

                        let [x, y, _] = v;

                        if x > 0 {
                            n.push([x - 1, y, 0]);
                        }

                        if x < ZONE_SIZE.0 - 1 {
                            n.push([x + 1, y, 0]);
                        }

                        if y > 0 {
                            n.push([x, y - 1, 0]);
                        }

                        if y < ZONE_SIZE.1 - 1 {
                            n.push([x, y + 1, 0]);
                        }

                        n
                    },
                    max_depth: 1000
                });

                if result.is_success {
                    // info!("path {}", result.path.iter().map(|[a, b, c]| format!("{},{}", a, b)).collect::<Vec<_>>().join(" "));
                    for [x, y, _] in result.path {
                        if x < ZONE_SIZE.0 && y < ZONE_SIZE.1 {
                            terrain.set(x, y, Terrain::River);
                        }
                    }
                } else {
                    info!("Failure!");
                }
            });
        }

        // every footpath should attempt to connect to every other footpath
        for (p1_idx, p1) in footpaths.iter().enumerate() {
            (p1_idx..footpaths.len()).for_each(|p2_idx| {
                let p2 = footpaths[p2_idx];

                if p1_idx == p2_idx {
                    return;
                }

                info!("start {},{}", p1.0, p1.1);
                info!("goal {},{}", p2.0, p2.1);

                // find path between p1/p2, and set terrain!
                let result = astar(AStarSettings {
                    start: [
                        p1.0,
                        p1.1,
                        0,
                    ],
                    is_goal: |p| {
                        p[0] == p2.0 && p[1] == p2.1
                    },
                    cost: |a, b| {
                        let dist = Distance::diagonal([a[0] as i32, a[1] as i32, a[2] as i32], [b[0] as i32, b[1] as i32, b[2] as i32]);
                        let t = terrain.get(b[0], b[1]).unwrap();

                        dist * match t {
                            Terrain::Grass => 5.,
                            Terrain::Dirt => 5.,
                            Terrain::River => 100.,
                            Terrain::Footpath => 1.,
                        }
                    },
                    heuristic: |v| {
                        Distance::diagonal([v[0] as i32, v[1] as i32, v[2] as i32], [p2.0 as i32, p2.1 as i32, 0])
                    },
                    neighbors: |v| {
                        let mut n = vec![];

                        let [x, y, _] = v;

                        if x > 0 {
                            n.push([x - 1, y, 0]);

                            if y > 0 {
                                n.push([x - 1, y - 1, 0]);
                            }

                            if y < ZONE_SIZE.1 - 1 {
                                n.push([x - 1, y + 1, 0]);
                            }
                        }

                        if x < ZONE_SIZE.0 - 1 {
                            n.push([x + 1, y, 0]);

                            if y > 0 {
                                n.push([x + 1, y - 1, 0]);
                            }

                            if y < ZONE_SIZE.1 - 1 {
                                n.push([x + 1, y + 1, 0]);
                            }
                        }

                        if y > 0 {
                            n.push([x, y - 1, 0]);
                        }

                        if y < ZONE_SIZE.1 - 1 {
                            n.push([x, y + 1, 0]);
                        }

                        n
                    },
                    max_depth: 1000
                });

                if result.is_success {
                    // info!("path {}", result.path.iter().map(|[a, b, c]| format!("{},{}", a, b)).collect::<Vec<_>>().join(" "));
                    for [x, y, _] in result.path {
                        if x < ZONE_SIZE.0 && y < ZONE_SIZE.1 {
                            terrain.set(x, y, Terrain::Footpath);
                        }
                    }
                } else {
                    info!("Failure!");
                }
            });
        }

        if ENABLE_ZONE_SNAPSHOTS {
            self.snapshots.push(terrain_snapshot(&terrain));
        }

        ZoneData {
            idx,
            terrain,
        }
    }
    
    fn get_snapshots(&self) -> Vec<ZoneSnapshot> {
        self.snapshots.to_vec()
    }
}
