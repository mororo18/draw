use std::ops::{
    Add,
    Sub,
    Div,
    Mul,
};

pub const EPS: f32 = 0.0000001;

#[derive(Debug, Copy, Clone)]
pub
struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub
    fn new (x: f32, y: f32) -> Self {Vec2 {x:x, y:y}}
}

impl Vec2 {
    pub
    fn dist (self, arg: Self) -> f32 {
        let x_sq = (self.x - arg.x).powi(2);
        let y_sq = (self.y - arg.y).powi(2);
        
        (x_sq + y_sq).sqrt()
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add (self, rhs: Self) -> Self {
        Vec2::new(
            self.x + rhs.x,
            self.y + rhs.y,
        )
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub (self, rhs: Self) -> Self {
        self + Vec2::new( -rhs.x, -rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul (self, rhs: f32) -> Self {
        Self {
			x: self.x * rhs,
			y: self.y * rhs,
        }
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;

    fn div (self, rhs: f32) -> Self {
        Self {
			x: self.x / rhs,
			y: self.y / rhs,
        }
    }
}



#[derive(Debug, Copy, Clone)]
pub
struct Vec4 {
    a: [f32; 4],
}

impl Vec4 {
    pub
    fn new (data: [f32; 4]) -> Self {
        Self {a: data}
    }

    pub
    fn zeros () -> Self {
        Self::new([0., 0., 0., 0.])
    }

    pub
    fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.a[0], self.a[1])
    }

    pub
    fn as_vec3(&self) -> Vec3 {
        Vec3::new([
            self.a[0], 
            self.a[1],
            self.a[2]
        ])
    }

    pub
    fn vec3_over_w(&self) -> Vec3 {
        self.as_vec3() / self.get_w()
    }

	pub fn x(&self) -> f32 {self.a[0]}
	pub fn get_y(&self) -> f32 {self.a[1]}
	pub fn get_z(&self) -> f32 {self.a[2]}
	pub fn get_w(&self) -> f32 {self.a[3]}

}

impl Add for Vec4 {
    type Output = Self;

    fn add (self, rhs: Self) -> Self {
        Self {
            a: [
            self.a[0] + rhs.a[0],
            self.a[1] + rhs.a[1],
            self.a[2] + rhs.a[2],
            self.a[3] + rhs.a[3]
            ],
        }
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;

    fn mul (self, rhs: f32) -> Self {
        Self {
            a: [
            self.a[0] * rhs,
            self.a[1] * rhs,
            self.a[2] * rhs,
            self.a[2] * rhs
            ],
        }
    }
}

impl Mul<Vec4> for f32 {
    type Output = Vec4;

    fn mul (self, rhs: Vec4) -> Vec4 {
        rhs * self
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub
struct Vec3 {
    a: [f32; 3],
}

impl Vec3 {
    pub
    fn new (data: [f32; 3]) -> Self {
        Self {a: data}
    }

    pub
    fn zeros () -> Self {
        Self {a: [0.0; 3]}
    }

    pub
    fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.a[0], self.a[1])
    }

    pub
    fn as_vec4(&self) -> Vec4 {
        Vec4::new([
            self.a[0], 
            self.a[1], 
            self.a[2], 
            1.0
        ])
    }

    pub
    fn norm (&self) -> f32 {
        let sum = self.a[0].powi(2) +
                  self.a[1].powi(2) +
                  self.a[2].powi(2);

        sum.sqrt()
    }

    pub
    fn normalized (&self) -> Self {
       *self / self.norm()
    }


    pub
    fn dist (&self, arg: Self) -> f32 {
        let diff = *self - arg;
        diff.norm()
    }

    pub
    fn dot (&self, arg: Self) -> f32 {
		self.x() * arg.x() +
		self.y() * arg.y() +
		self.z() * arg.z()
    }

    pub
    fn cross(&self, arg: Self) -> Self {
        let a_0 = self.x();
        let a_1 = self.y();
        let a_2 = self.z();

        let b_0 = arg.x();
        let b_1 = arg.y();
        let b_2 = arg.z();

        let c_0 = (a_1 * b_2) - (a_2 * b_1);
        let c_1 = (a_2 * b_0) - (a_0 * b_2);
        let c_2 = (a_0 * b_1) - (a_1 * b_0);

        Self::new([c_0, c_1, c_2])
    }

    pub fn x (&self) -> f32 {self.a[0]}
    pub fn y (&self) -> f32 {self.a[1]}
    pub fn z (&self) -> f32 {self.a[2]}

}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul (self, rhs: f32) -> Self {
        Self {
            a: [
            self.a[0] * rhs,
            self.a[1] * rhs,
            self.a[2] * rhs
            ],
        }
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div (self, rhs: f32) -> Self {
        Self {
            a: [
            self.a[0] / rhs,
            self.a[1] / rhs,
            self.a[2] / rhs
            ],
        }
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add (self, rhs: Self) -> Self {
        Self {
            a: [
            self.a[0] + rhs.a[0],
            self.a[1] + rhs.a[1],
            self.a[2] + rhs.a[2]
            ],
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub (self, rhs: Self) -> Self {
        Self {
            a: [
            self.a[0] - rhs.a[0],
            self.a[1] - rhs.a[1],
            self.a[2] - rhs.a[2]
            ],
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub
struct Matrix4 {
    a: [[f32; 4]; 4],
}

impl Matrix4 {
    pub
    fn new (data: [[f32; 4]; 4]) -> Self {
        Self {a: data}
    }

    pub
    fn zeros () -> Self {
        Self {a: [[0.0; 4]; 4]}
    }

	pub
	fn transposed(&self) -> Self {
        let mut cpy = self.clone(); 

        for i in 0..4 {
            for j in i+1..4 {
                let tmp = cpy.a[i][j];
                cpy.a[i][j] = cpy.a[j][i];
                cpy.a[j][i] = tmp;
            }
        }

        cpy
	}

    pub
    fn rotate_x (theta: f32) -> Self {
        let sin = theta.sin();
        let cos = theta.cos();

        Self::new([
            [1.,    0.,     0.,     0.],
            [0.,   cos,   -sin,     0.],
            [0.,   sin,    cos,     0.],
            [0.,    0.,     0.,     1.]
        ])
    }

    pub
    fn rotate_y (theta: f32) -> Self {
        let sin = theta.sin();
        let cos = theta.cos();

        Self::new([
            [cos,    0.,   sin,     0.],
            [0.,     1.,    0.,     0.],
            [-sin,   0.,   cos,     0.],
            [0.,     0.,    0.,     1.]
        ])
    }

    pub
    fn rotate_z (theta: f32) -> Self {
        let sin = theta.sin();
        let cos = theta.cos();

        Self::new([
            [cos,  -sin,    0.,     0.],
            [sin,   cos,    0.,     0.],
            [0.,     0.,    1.,     0.],
            [0.,     0.,    0.,     1.]
        ])
    }

}

// TODO: ver ->
// https://medium.com/@kilichbekhaydarov/toward-an-optimal-matrix-multiplication-algorithm-4f024baa1206
impl Mul for Matrix4 {
    type Output = Self;

    fn mul (self, rhs: Self) -> Self {
        let mut c = Self::zeros();

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    c.a[i][j] += self.a[i][k] * rhs.a[k][j];
                }
            }
        }

        c
    }
}

impl Mul<Vec4> for Matrix4 {
    type Output = Vec4;

    fn mul (self, rhs: Vec4) -> Vec4 {
        let mut out = Vec4::zeros();

        for i in 0..4 {
            for j in 0..4 {
                out.a[i] += self.a[i][j] * rhs.a[j];
            }
        }

        out
    }
}

