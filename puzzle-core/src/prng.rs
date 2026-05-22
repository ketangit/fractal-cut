/// Replicates the JS PRNG exactly:
///   var x = Math.sin(seed) * 10000; seed += 1; return x - Math.floor(x);
/// seed is f64 to match IEEE 754 double arithmetic.
pub struct Prng {
    pub seed: f64,
}

impl Prng {
    pub fn new(seed: f64) -> Self {
        Prng { seed }
    }

    pub fn random(&mut self) -> f64 {
        let x = self.seed.sin() * 10000.0;
        self.seed += 1.0;
        x - x.floor()
    }

    pub fn uniform(&mut self, min: f64, max: f64) -> f64 {
        let r = self.random();
        min + r * (max - min)
    }

    pub fn rbool(&mut self) -> bool {
        self.random() > 0.5
    }

    /// Capture current seed so coloring can be replayed from this point.
    pub fn snapshot(&self) -> f64 {
        self.seed
    }

    pub fn restore(&mut self, snapshot: f64) {
        self.seed = snapshot;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_golden(seed: f64, expected: &[f64]) {
        let mut prng = Prng::new(seed);
        for (i, &exp) in expected.iter().enumerate() {
            let got = prng.random();
            // GNU libm (Rust) and V8's libm (JS) may differ by up to ~2 ULP for some
            // sin() arguments (e.g. sin(18)). 1e-10 catches real bugs while tolerating
            // platform-specific transcendental precision.
            assert!(
                (got - exp).abs() < 1e-10,
                "seed={seed} draw {i}: got {got}, expected {exp}"
            );
        }
    }

    #[test]
    fn golden_seed_0() {
        check_golden(
            0.0,
            &[
                0.0,
                0.7098480789645691,
                0.9742682568175951,
                0.20008059867222983,
                0.9750469207183414,
                0.7572533686161478,
                0.845018010741569,
                0.8659871878908234,
                0.5824662338181952,
                0.18485241756570758,
                0.7888911063018895,
                0.0979344929655781,
                0.27081999565052683,
                0.6703682664092412,
                0.07355694870420848,
                0.8784015711680695,
                0.9668333493468708,
                0.025081204432353843,
                0.12753228323799704,
                0.772096629523503,
            ],
        );
    }

    #[test]
    fn golden_seed_1() {
        check_golden(
            1.0,
            &[
                0.7098480789645691,
                0.9742682568175951,
                0.20008059867222983,
                0.9750469207183414,
                0.7572533686161478,
                0.845018010741569,
                0.8659871878908234,
                0.5824662338181952,
                0.18485241756570758,
                0.7888911063018895,
                0.0979344929655781,
                0.27081999565052683,
                0.6703682664092412,
                0.07355694870420848,
                0.8784015711680695,
                0.9668333493468708,
                0.025081204432353843,
                0.12753228323799704,
                0.772096629523503,
                0.45250727627717424,
            ],
        );
    }

    #[test]
    fn golden_seed_42() {
        check_golden(
            42.0,
            &[
                0.7845208436629036,
                0.2525737140167621,
                0.01925105413576489,
                0.03524534118514566,
                0.8834764880921284,
                0.7312274522400912,
                0.4533867633317641,
                0.4734724052814272,
                0.25146296071216057,
                0.29175843374741817,
                0.27592040485251346,
                0.25150181834169416,
                0.10951148383719556,
                0.4482664138013206,
                0.489979130881693,
                0.6475524782490538,
                0.726480845371043,
                0.38007139137880586,
                0.8937889778330828,
                0.8222999160698237,
            ],
        );
    }

    #[test]
    fn golden_seed_999() {
        check_golden(
            999.0,
            &[
                0.3924726293587355,
                0.7954053200246562,
                0.9059758632192825,
                0.665419737925049,
                0.5839905973816713,
                0.05574862850335194,
                0.7550976434804397,
                0.07735679517281824,
                0.6186620027601748,
                0.8337575358982576,
                0.7184074057795442,
                0.42829739172157133,
                0.8595045915899391,
                0.08191685142764982,
                0.42327761150954757,
                0.6205782777460627,
                0.37882538217581896,
                0.20167636954101908,
                0.03232029219634569,
                0.6286217291594767,
            ],
        );
    }

    #[test]
    fn golden_seed_9999() {
        check_golden(
            9999.0,
            &[
                0.8695639623356328,
                0.8561111174785765,
                0.6472555815635133,
                0.8803484295267481,
                0.8777860954214702,
                0.5645436019894987,
                0.5365041333934641,
                0.05111201564869816,
                0.4330641306914913,
                0.4413826186009828,
                0.5369010176659685,
                0.24953771777563816,
                0.9348638767132798,
                0.06872427555435934,
                0.06906758517652634,
                0.9902705739514204,
                0.9649931498138358,
                0.03205344289290224,
                0.9365832696294092,
                0.5425124499215599,
            ],
        );
    }

    #[test]
    fn snapshot_restore() {
        let mut prng = Prng::new(10.0);
        for _ in 0..5 {
            prng.random();
        }
        let snap = prng.snapshot();
        let v1 = prng.random();
        prng.restore(snap);
        let v2 = prng.random();
        assert_eq!(v1, v2);
    }

    #[test]
    fn seed_increments_by_one() {
        let mut prng = Prng::new(0.0);
        prng.random();
        assert_eq!(prng.seed, 1.0);
        prng.random();
        assert_eq!(prng.seed, 2.0);
    }

    #[test]
    fn uniform_range() {
        let mut prng = Prng::new(7.0);
        for _ in 0..50 {
            let v = prng.uniform(3.0, 9.0);
            assert!(v >= 3.0 && v < 9.0);
        }
    }
}
