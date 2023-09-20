use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, SimpleFloorPlanner, Value},
    dev::MockProver,
    pasta::Fp,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Fixed, Instance, Selector},
    poly::Rotation,
};
use plotters::{
    prelude::{BitMapBackend, IntoDrawingArea},
    style::WHITE,
};

// chip에 들어가야 할 trait interfaces. 공식문서에서는 instructions라고 표기 됨.
trait Ops {
    type Num;
    fn load_private(&self, layouter: impl Layouter<Fp>, x: Value<Fp>) -> Result<Self::Num, Error>;
    fn load_constant(&self, layouter: impl Layouter<Fp>, constant: Fp) -> Result<Self::Num, Error>;
    // multiplication ( a * b = c )
    fn mul(
        &self,
        layouter: impl Layouter<Fp>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error>;
    // addition
    fn add(
        &self,
        layouter: impl Layouter<Fp>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error>;
    fn expose_public(
        &self,
        layouter: impl Layouter<Fp>,
        num: Self::Num,
        row: usize,
    ) -> Result<(), Error>;
}

// codes for chip
struct MyChip {
    config: MyConfig,
}
// halo2 crate에 정의된 Chip trait의 interface 여기서 정의함.
impl Chip<Fp> for MyChip {
    type Config = MyConfig;

    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}
// 위에서 정의한 Ops trait란 interface를 여기서 정의함.
impl Ops for MyChip {
    type Num = AssignedCell<Fp, Fp>;

    fn load_private(
        &self,
        mut layouter: impl Layouter<Fp>,
        value: Value<Fp>,
    ) -> Result<Self::Num, Error> {
        let config = self.config();

        layouter.assign_region(
            || "load private",
            |mut region| region.assign_advice(|| "private input", config.advice[0], 0, || value),
        )
    }
    // layouter에 assign 해야하니까 mutable하게 넣어줘야 함
    fn load_constant(
        &self,
        mut layouter: impl Layouter<Fp>,
        value: Fp,
    ) -> Result<Self::Num, Error> {
        let config = self.config();

        layouter.assign_region(
            || "load constant",
            |mut region| -> Result<AssignedCell<Fp, _>, Error> {
                region.assign_advice_from_constant(|| "constant value", config.advice[0], 0, value)
            },
        )
    }

    fn mul(
        &self,
        mut layouter: impl Layouter<Fp>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        let config = self.config();
        layouter.assign_region(
            || "mul",
            |mut region| {
                // turn on multiplication selector
                config.s_mul.enable(&mut region, 0)?;
                // computation
                a.copy_advice(|| "lhs", &mut region, config.advice[0], 0)?;
                b.copy_advice(|| "rhs", &mut region, config.advice[1], 0)?;
                // make new value
                let value = a.value().and_then(|a| b.value().map(|b| *a * *b));
                // assgin new value into our config
                region.assign_advice(|| "a * b", config.advice[0], 1, || value)
            },
        )
    }

    fn add(
        &self,
        mut layouter: impl Layouter<Fp>,
        a: Self::Num,
        b: Self::Num,
    ) -> Result<Self::Num, Error> {
        let config = self.config();
        layouter.assign_region(
            || "add",
            |mut region| {
                // turn on multiplication selector
                config.s_add.enable(&mut region, 0)?;
                // computation
                a.copy_advice(|| "lhs", &mut region, config.advice[0], 0)?;
                b.copy_advice(|| "rhs", &mut region, config.advice[1], 0)?;
                // make new value
                let value = a.value().and_then(|a| b.value().map(|b| *a + *b));
                // assgin new value into our config
                region.assign_advice(|| "a + b", config.advice[0], 1, || value)
            },
        )
    }

    fn expose_public(
        &self,
        mut layouter: impl Layouter<Fp>,
        num: Self::Num,
        row: usize,
    ) -> Result<(), Error> {
        let config = self.config();
        layouter.constrain_instance(num.cell(), config.instance, row)
    }
}

impl MyChip {
    fn new(config: MyConfig) -> Self {
        MyChip { config }
    }

    fn configure(
        meta: &mut ConstraintSystem<Fp>,
        advice: [Column<Advice>; 2],
        instance: Column<Instance>,
        constant: Column<Fixed>,
    ) -> MyConfig {
        meta.enable_equality(instance);
        meta.enable_constant(constant);

        for column in advice.iter() {
            meta.enable_equality(*column);
        }

        let s_mul = meta.selector();
        let s_add = meta.selector();

        meta.create_gate("mul/add", |meta| {
            let lhs = meta.query_advice(advice[0], Rotation::cur());
            let rhs = meta.query_advice(advice[1], Rotation::cur());
            let out = meta.query_advice(advice[0], Rotation::next());

            let s_mul = meta.query_selector(s_mul);
            let s_add = meta.query_selector(s_add);

            vec![
                s_mul * (lhs.clone() * rhs.clone() - out.clone()),
                s_add * (lhs + rhs - out),
            ]
        });

        MyConfig {
            advice,
            instance,
            s_mul,
            s_add,
        }
    }
}

// codes for config
#[derive(Clone, Debug)]
struct MyConfig {
    advice: [Column<Advice>; 2],
    instance: Column<Instance>,
    s_mul: Selector,
    s_add: Selector,
}

// codes for circuit
#[derive(Default)]
struct MyCircuit {
    x: Value<Fp>,
    constant: Fp,
}

impl Circuit<Fp> for MyCircuit {
    type Config = MyConfig;
    type FloorPlanner = SimpleFloorPlanner; // -> what is meaning?

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let advice = [meta.advice_column(), meta.advice_column()];
        let instance = meta.instance_column();
        let constant = meta.fixed_column();

        MyChip::configure(meta, advice, instance, constant)
    }

    // 여기서 실제 서킷을 짜면 됨.
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let chip = MyChip::new(config);
        let x = chip.load_private(layouter.namespace(|| "load x"), self.x)?;
        let constant = chip.load_constant(layouter.namespace(|| "load constant"), self.constant)?;

        // x2 = x * x
        let x2 = chip.mul(layouter.namespace(|| "x2"), x.clone(), x.clone())?;

        // x3 = x2 * x
        let x3 = chip.mul(layouter.namespace(|| "x3"), x2, x.clone())?;

        // x3_x = x3 + x
        let x3_x = chip.add(layouter.namespace(|| "x3+x"), x3, x)?;

        // x3_x_5 = x3_x + 5
        let x3_x_5 = chip.add(layouter.namespace(|| "x3+x+5"), x3_x, constant)?;

        chip.expose_public(layouter.namespace(|| "expose res"), x3_x_5, 0)
    }
}

// original expression
// x^3 + x + 5 = 35, and we need to change form like that.

// x2 = x * x
// x3 = x2 * x
// x3_x = x3 + x
// x3_x_5 = x3_x + 5
// x3_x_5 == 35

// | a      | b | m | s |
// | x      | x | 1 | 0 |
// | x2     | x | 1 | 0 |
// | x3     | x | 0 | 1 |
// | x3+x   | 5 | 0 | 1 |
// | x3+x+5 |

fn main() {
    let k = 4;
    let x = Fp::from(3);
    let constant = Fp::from(5);
    let res = Fp::from(35);

    let circuit = MyCircuit {
        x: Value::known(x),
        constant,
    };

    let public_inputs = vec![res];
    // let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
    // assert_eq!(prover.verify(), Ok(()));

    let root = BitMapBackend::new("layout.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root
        .titled("Example Circuit Layout", ("sans-serif", 60))
        .unwrap();

    halo2_proofs::dev::CircuitLayout::default()
        // You can optionally render only a section of the circuit.
        .view_width(0..2)
        .view_height(0..16)
        // You can hide labels, which can be useful with smaller areas.
        .show_labels(false)
        // Render the circuit onto your area!
        // The first argument is the size parameter for the circuit.
        .render(5, &circuit, &root)
        .unwrap();
}
