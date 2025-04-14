pub struct Prize {
    name: String,
    count: usize,
}

pub struct PrizeBuilder {
    name: String,
    count: usize,
}

impl Prize {
    pub fn new(name: String, count: usize) -> Self {
        Self {
            name,
            count,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl PrizeBuilder {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            count: 1,
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn build(self) -> Prize {
        Prize {
            name: self.name,
            count: self.count,
        }
    }
}
