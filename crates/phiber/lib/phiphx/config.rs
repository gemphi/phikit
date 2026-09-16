//! Configuration for mesh to point cloud conversion.

/// Sampling method for point extraction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SamplingMethod {
    /// Uniform random sampling across all triangles
    Uniform,
    /// Area-weighted sampling (more points on larger triangles)
    AreaWeighted,
    /// BVH-accelerated spatial sampling with stratification
    BvhStratified,
}

impl Default for SamplingMethod {
    fn default() -> Self {
        SamplingMethod::AreaWeighted
    }
}

/// Configuration for mesh to point cloud conversion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshToCloudConfig {
    /// Number of points to sample per unit area
    pub density: f32,
    /// Minimum number of points to generate
    pub min_points: usize,
    /// Maximum number of points to generate
    pub max_points: usize,
    /// Include vertex normals in output
    pub include_normals: bool,
    /// Include vertex colors if available
    pub include_colors: bool,
    /// Seed for random sampling (0 = random)
    pub seed: u64,
    /// Use uniform sampling (vs. area-weighted)
    /// Deprecated: use `sampling_method` instead
    pub uniform_sampling: bool,
    /// Sampling method for point extraction
    pub sampling_method: SamplingMethod,
    /// BVH depth for spatial partitioning (0 = auto)
    pub bvh_depth: usize,
    /// Enable BVH-based spatial queries
    pub use_bvh: bool,
}

impl Default for MeshToCloudConfig {
    fn default() -> Self {
        Self {
            density: 100.0,
            min_points: 100,
            max_points: 1_000_000,
            include_normals: true,
            include_colors: true,
            seed: 0,
            uniform_sampling: false,
            sampling_method: SamplingMethod::default(),
            bvh_depth: 0,   // Auto
            use_bvh: false, // Disabled by default
        }
    }
}

impl MeshToCloudConfig {
    /// Create default config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set point density
    pub fn with_density(mut self, density: f32) -> Self {
        self.density = density.max(0.1);
        self
    }

    /// Set point count limits
    pub fn with_limits(mut self, min: usize, max: usize) -> Self {
        self.min_points = min;
        self.max_points = max;
        self
    }

    /// Set normal inclusion
    pub fn with_normals(mut self, include: bool) -> Self {
        self.include_normals = include;
        self
    }

    /// Set color inclusion
    pub fn with_colors(mut self, include: bool) -> Self {
        self.include_colors = include;
        self
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Enable uniform sampling
    pub fn uniform(mut self) -> Self {
        self.uniform_sampling = true;
        self
    }

    /// Enable area-weighted sampling (default)
    pub fn area_weighted(mut self) -> Self {
        self.uniform_sampling = false;
        self.sampling_method = SamplingMethod::AreaWeighted;
        self
    }

    /// Enable BVH-stratified sampling
    pub fn bvh_stratified(mut self, depth: usize) -> Self {
        self.sampling_method = SamplingMethod::BvhStratified;
        self.use_bvh = true;
        self.bvh_depth = depth;
        self
    }

    /// Enable BVH for spatial queries
    pub fn with_bvh(mut self, depth: usize) -> Self {
        self.use_bvh = true;
        self.bvh_depth = depth;
        self
    }

    /// Set sampling method directly
    pub fn with_sampling_method(mut self, method: SamplingMethod) -> Self {
        self.sampling_method = method;
        self
    }
}
