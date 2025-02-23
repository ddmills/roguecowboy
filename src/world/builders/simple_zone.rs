use bevy::log::info;

use crate::{
    common::{astar, remap, AStarSettings, Distance, Grid, Perlin, Rand},
    projection::ZONE_SIZE,
    world::Terrain,
};

use super::{
    bool_snapshot, edge_gradient_buffer, edge_snapshot, grayscale_snapshot, noise_grid, rand_grid, terrain_snapshot, ZoneBuilder, ZoneConstraints, ZoneData, ZoneSnapshot, ENABLE_ZONE_SNAPSHOTS
};

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

        let height = noise_grid(idx as u32, 0.1, 2, 2.);

        if ENABLE_ZONE_SNAPSHOTS {
            // self.snapshots.push(grayscale_snapshot(&height));
        }

        let edge_buffer = edge_gradient_buffer(8, 1.);
        let rand_noise = rand_grid(idx as u32);

        if ENABLE_ZONE_SNAPSHOTS {
            // self.snapshots.push(grayscale_snapshot(&edge_buffer));
            // let mul = edge_buffer.map(|x, y, v| {
            //     remap(1. - v, 0.5, 1.)
            // });
            // self.snapshots.push(grayscale_snapshot(&mul));
            self.snapshots.push(bool_snapshot(&rand_noise));
        }

        // every river should attempt to connect to every other river,
        // and also follow low ground
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
                    start: [p1.0, p1.1],
                    is_goal: |p| p[0] == p2.0 && p[1] == p2.1,
                    cost: |a, b| {
                        let [x, y] = b;
                        let h = height.get(x, y).unwrap();
                        let r = rand_noise.get(x, y).unwrap();
                        let t = terrain.get(x, y).unwrap();
                        let e = remap(1. - edge_buffer.get(x, y).unwrap(), 0.25, 1.);
            
                        let terrain_cost: f32 = match t {
                            Terrain::Grass => 1.0,
                            Terrain::Dirt => 1.0,
                            Terrain::River => 0.001,
                            Terrain::Footpath => 1.0,
                        };
            
                        let rand_cost = match r {
                            true => 10.0,
                            false => 1.0,
                        };
            
                        rand_cost * h * terrain_cost
                    },
                    heuristic: |[x, y]| {
                        0.1 * Distance::chebyshev(
                            [x as i32, y as i32, 0],
                            [p2.0 as i32, p2.1 as i32, 0],
                        )
                    },
                    neighbors: |v| {
                        let mut n = vec![];

                        let [x, y] = v;

                        if x > 0 {
                            n.push([x - 1, y]);

                            if y > 0 {
                                n.push([x - 1, y - 1]);
                            }

                            if y < ZONE_SIZE.1 - 1 {
                                n.push([x - 1, y + 1]);
                            }
                        }

                        if x < ZONE_SIZE.0 - 1 {
                            n.push([x + 1, y]);

                            if y > 0 {
                                n.push([x + 1, y - 1]);
                            }

                            if y < ZONE_SIZE.1 - 1 {
                                n.push([x + 1, y + 1]);
                            }
                        }

                        if y > 0 {
                            n.push([x, y - 1]);
                        }

                        if y < ZONE_SIZE.1 - 1 {
                            n.push([x, y + 1]);
                        }

                        n
                    },
                    max_depth: 10000,
                });

                if result.is_success {
                    // info!("path {}", result.path.iter().map(|[a, b, c]| format!("{},{}", a, b)).collect::<Vec<_>>().join(" "));
                    for [x, y] in result.path {
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
                    start: [p1.0, p1.1],
                    is_goal: |p| p[0] == p2.0 && p[1] == p2.1,
                    cost: |a, b| {
                        let [x, y] = b;
                        let t = terrain.get(x, y).unwrap();
                        let h = height.get(x, y).unwrap();
                        let edge = 1. - edge_buffer.get(x, y).unwrap();
                        let r = rand_noise.get(x, y).unwrap();

                        // info!("h at {} {} = {}", b[0], b[1], h);

                        let rand_cost = match r {
                            true => 4.0,
                            false => 1.0,
                        };

                        let terrain_cost = match t {
                            Terrain::Grass => 1.,
                            Terrain::Dirt => 1.,
                            Terrain::River => 20.,
                            Terrain::Footpath => 0.01,
                        };

                        // if *t == Terrain::Footpath {
                        //     return 1.;
                        // }

                        // if *t == Terrain::River {
                        //     // try to cross at right angles for rivers
                        //     if a[0] != b[0] && a[1] != b[1] {
                        //         return 100.;
                        //     }

                        //     return 10.;
                        // }

                        rand_cost * terrain_cost

                        // 1. + (h * h * h * 400.) + (edge * edge * 500.)
                    },
                    heuristic: |[x, y]| {
                        0.1 * Distance::chebyshev(
                            [x as i32, y as i32, 0],
                            [p2.0 as i32, p2.1 as i32, 0],
                        )
                    },
                    neighbors: |v| {
                        let mut n = vec![];

                        let [x, y] = v;

                        if x > 0 {
                            n.push([x - 1, y]);

                            if y > 0 {
                                n.push([x - 1, y - 1]);
                            }

                            if y < ZONE_SIZE.1 - 1 {
                                n.push([x - 1, y + 1]);
                            }
                        }

                        if x < ZONE_SIZE.0 - 1 {
                            n.push([x + 1, y]);

                            if y > 0 {
                                n.push([x + 1, y - 1]);
                            }

                            if y < ZONE_SIZE.1 - 1 {
                                n.push([x + 1, y + 1]);
                            }
                        }

                        if y > 0 {
                            n.push([x, y - 1]);
                        }

                        if y < ZONE_SIZE.1 - 1 {
                            n.push([x, y + 1]);
                        }

                        n
                    },
                    max_depth: 10000,
                });

                if result.is_success {
                    // info!("path {}", result.path.iter().map(|[a, b, c]| format!("{},{}", a, b)).collect::<Vec<_>>().join(" "));
                    for [x, y] in result.path {
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

        ZoneData { idx, terrain }
    }

    fn get_snapshots(&self) -> Vec<ZoneSnapshot> {
        self.snapshots.to_vec()
    }
}
