use rand::RngExt;

const MUTATION_RATE: f32 = 0.1;
const MUTATION_SCALE: f32 = 0.3;

pub struct Network {
    pub layers: Vec<Layer>,
    pub size: usize,

    pub memories: Vec<f32>,
    pub memory_size: usize,
    pub memory_function: Function,
}

pub struct Layer {
    pub neurons: Vec<Neuron>,
    pub size: usize,
}

pub struct Neuron {
    pub bias: f32,
    pub weights: Vec<f32>,
    pub function: Function,

    pub size: usize,
}

#[derive(Clone)]
pub enum Function {
    Sigmoid,
    Relu,
    Tanh,
}

impl Network {
    pub fn new(sizes: Vec<usize>, function: Vec<Function>, memory_size: usize, memory_function: Function) -> Network {
        let size = sizes.len();
        Network { 
            layers: {
                let mut l  = Vec::new();
                for i in 1..size {
                    let mut prev_layer_size = sizes[i - 1];
                    if i == 1 {
                        prev_layer_size += memory_size;
                    }

                    let mut this_layer_size = sizes[i];
                    if i == size - 1 {
                        this_layer_size += memory_size
                    }

                    l.push(Layer::new(this_layer_size, prev_layer_size, &function[i - 1]));
                };
                l
            },
            size: size,
            memories: {
                let mut v  = Vec::new();
                for _i  in 0..memory_size {
                    v.push(0.0);
                };
                v
            }, 
            memory_size, 
            memory_function,
        }
    }
    pub fn run(&mut self, input: &Vec<f32>) -> Vec<f32> {
        let mut output = input.clone();

        for memory in &self.memories {
            output.push(run_function(&self.memory_function, memory.clone()));
        }

        for l in 0..self.layers.len(){
            output = self.layers[l].run(&output);
        };

        let memory_deltas = output.split_off(output.len() - self.memory_size);
        for m in 0..self.memories.len() {
            self.memories[m] += memory_deltas[m];
        }                                     

        output
    }
    pub fn breed(a: &Self, b: &Self) -> Self {
        let mut layers = Vec::new();
        let size = a.size.min(b.size);
        for l in 0..(size - 1) {
            layers.push(Layer::breed(&a.layers[l], &b.layers[l]));
        }
        Network {
            layers,
            size, 
            memories: {
                let mut v  = Vec::new();
                for _i  in 0..a.memory_size {
                    v.push(0.0);
                };
                v
            }, 
            memory_size: a.memory_size, 
            memory_function: a.memory_function.clone(),         
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
            memories: {
                let mut v  = Vec::new();
                for _i  in 0..self.memory_size{
                    v.push(0.0);
                };
                v
            }, 
            memory_size: self.memory_size, 
            memory_function: self.memory_function.clone(),         
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
        for i in 0..a.size.min(b.size) {
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
            function: func,
        }
    }
    fn run(&self, input: &Vec<f32>) -> f32 {
        let mut output = self.bias;
        for w in 0..self.weights.len() {
            output += input[w] * self.weights[w];
        };
        run_function(&self.function, output)
    }
    fn breed(a: &Self, b: &Self) -> Self {
        let mut weights = Vec::new();
        for i in 0..a.size.min(b.size) {
            weights.push(crossover(a.weights[i], b.weights[i]));
        }
        Neuron {
            bias: crossover(a.bias, b.bias),
            weights,
            size: a.size,
            function: a.function.clone(),
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
            function: self.function.clone(),
        }
    }
}

fn run_function(function: &Function, x: f32) -> f32 {
    match function {
        Function::Sigmoid => sigmoid(x),
        Function::Relu => relu(x),
        Function::Tanh => tanh(x),
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
    rng.random::<f32>() * 2.0 - 1.0
}