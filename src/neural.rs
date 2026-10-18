use rand::RngExt;

const MUTATION_RATE: f32 = 0.1;
const MUTATION_SCALE: f32 = 0.5;

pub struct Network {
    pub layers: Vec<Layer>,
    pub size: usize,
}

pub struct Layer {
    pub neurons: Vec<Neuron>,
    pub size: usize,
}

pub struct Neuron {
    pub bias: f32,
    pub weights: Vec<f32>,
    pub size: usize,
    pub func: Function
}

#[derive(Clone)]
pub enum Function {
    Sigmoid,
    Relu,
    Tanh,
}

impl Network {
    pub fn new(sizes: Vec<usize>, funcs: Vec<Function>) -> Network {
        let size = sizes.len();
        Network { 
            layers: {
                let mut l  = Vec::new();
                for i in 1..size {
                    l.push(Layer::new(sizes[i], sizes[i - 1], &funcs[i - 1]));
                };
                l
            },
            size: size,
        }
    }
    pub fn run(&self, input: &Vec<f32>) -> Vec<f32>{
        let mut output = input.clone();
        for l in 0..self.layers.len(){
            output = self.layers[l].run(&output);
        };
        output
    }
    pub fn breed(a: &Self, b: &Self) -> Self {
        let mut layers = Vec::new();
        for l in 0..(a.size - 1) {
            layers.push(Layer::breed(&a.layers[l], &b.layers[l]));
        }
        Network {
            layers,
            size: a.size,            
        }
    }
    pub fn vary(&self) -> Self {
        let mut layers = Vec::new();
        for l in 0..(self.size - 1) {
            layers.push(self.layers[l].vary());
        }
        Network {
            layers,
            size: self.size,            
        }
    }
}

impl Layer {
    fn new(size: usize, prev_layer_size: usize, func: &Function) -> Layer {
        Layer {
            neurons: {
                let mut n = Vec::new();
                for _i in 0..size {
                    n.push(Neuron::new(prev_layer_size, func.clone()));
                };
                n
            },
            size: size,
        }
    }
    fn run(&self, input: &Vec<f32>) -> Vec<f32>{
        let mut output = Vec::new();
        for n in 0..self.neurons.len() {
            output.push(self.neurons[n].run(&input));
        };
        output
    }
    fn breed(a: &Self, b: &Self) -> Self {
        let mut neurons = Vec::new();
        for i in 0..a.size {
            neurons.push(Neuron::breed(&a.neurons[i], &b.neurons[i]));
        }
        Layer {
            neurons,
            size: a.size,
        }
    }
    fn vary(&self) -> Self {
        let mut neurons = Vec::new();
        for n in 0..self.size {
            neurons.push(self.neurons[n].vary());
        }
        Layer {
            neurons,
            size: self.size,
        }
    }
}

impl Neuron {
    fn new(prev_layer_size: usize, func: Function) -> Neuron {    
        Neuron { 
            bias: ran_param(), 
            weights: {
                let mut w = Vec::new();
                for _i in 0..prev_layer_size {
                    w.push(ran_param());
                };
                w
            },
            size: prev_layer_size,
            func: func,
        }
    }
    fn run(&self, input: &Vec<f32>) -> f32{
        let mut output = self.bias;
        for w in 0..self.weights.len() {
            output += input[w] * self.weights[w];
        };
        match self.func {
            Function::Sigmoid => sigmoid(output),
            Function::Relu => relu(output),
            Function::Tanh => tanh(output),
        }
    }
    fn breed(a: &Self, b: &Self) -> Self {
        let mut weights = Vec::new();
        for i in 0..a.size {
            weights.push(crossover(a.weights[i], b.weights[i]));
        }
        Neuron {
            bias: crossover(a.bias, b.bias),
            weights,
            size: a.size,
            func: a.func.clone(),
        }
    }
    fn vary(&self) -> Self {
        let mut weights = Vec::new();
        for i in 0..self.size {
            weights.push(vary(self.weights[i]));
        }
        Neuron {
            bias: vary(self.bias),
            weights,
            size: self.size,
            func: self.func.clone(),
        }
    }
}


fn sigmoid(x: f32) -> f32 {
    1.0 - (1.0 / (1.0 + x.exp()))
}
fn relu(x: f32) -> f32 {
    x.max(0.0)
}
fn tanh(x: f32) -> f32 {
    x.tanh()
}

fn vary(a: f32) -> f32 {
    let mut rng = rand::rng();
    if rng.random::<f32>() < MUTATION_RATE {
        a + rng.random_range(-MUTATION_SCALE..MUTATION_SCALE)
    } else {
        a
    }
}
fn crossover(a: f32, b: f32) -> f32 {
    let mut rng = rand::rng();
    let pick = if rng.random::<bool>() { a } else { b };
    if rng.random::<f32>() < MUTATION_RATE {
        pick + rng.random_range(-MUTATION_SCALE..MUTATION_SCALE)
    } else {
        pick
    }
}
fn ran_param() -> f32 {
    let mut rng = rand::rng();
    ((1.0 / (1.0 - rng.random::<f32>())) - 1.0).ln()
}