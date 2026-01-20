//! Resource estimation for execution plans
//!
//! This module provides functionality for estimating resource requirements
//! of execution plans including CPU, memory, disk, and network usage.

use crate::types::*;
use crate::error::PlanningError;
use crate::planner::{CpuUsage, MemoryUsage, DiskIO, NetworkIO};
use std::collections::HashMap;
use tracing::debug;

/// Resource estimator for execution plans
pub struct ResourceEstimator {
    /// Configuration for resource estimation
    config: ResourceEstimationConfig,
    /// Historical resource usage data
    historical_data: HashMap<String, ResourceProfile>,
    /// System resource limits
    system_limits: SystemResourceLimits,
}

/// Configuration for resource estimation
#[derive(Debug, Clone)]
pub struct ResourceEstimationConfig {
    /// Whether to use historical data for estimation
    pub use_historical_data: bool,
    /// Safety margin for resource estimates (0.0 to 1.0)
    pub safety_margin: f32,
    /// Whether to consider parallel execution
    pub consider_parallel_execution: bool,
    /// Default resource multipliers
    pub default_multipliers: ResourceMultipliers,
}

impl Default for ResourceEstimationConfig {
    fn default() -> Self {
        Self {
            use_historical_data: true,
            safety_margin: 0.2, // 20% safety margin
            consider_parallel_execution: true,
            default_multipliers: ResourceMultipliers::default(),
        }
    }
}

/// Resource multipliers for different operation types
#[derive(Debug, Clone)]
pub struct ResourceMultipliers {
    /// CPU multiplier for different command types
    pub cpu_multipliers: HashMap<String, f32>,
    /// Memory multiplier for different command types
    pub memory_multipliers: HashMap<String, f32>,
    /// Disk I/O multiplier for different command types
    pub disk_multipliers: HashMap<String, f32>,
    /// Network I/O multiplier for different command types
    pub network_multipliers: HashMap<String, f32>,
}

impl Default for ResourceMultipliers {
    fn default() -> Self {
        let mut cpu_multipliers = HashMap::new();
        cpu_multipliers.insert("execute_".to_string(), 0.3);
        cpu_multipliers.insert("perform_analysis".to_string(), 0.6);
        cpu_multipliers.insert("validate_".to_string(), 0.1);
        cpu_multipliers.insert("backup_".to_string(), 0.2);
        cpu_multipliers.insert("apply_config".to_string(), 0.15);

        let mut memory_multipliers = HashMap::new();
        memory_multipliers.insert("execute_".to_string(), 200.0); // 200MB
        memory_multipliers.insert("perform_analysis".to_string(), 500.0); // 500MB
        memory_multipliers.insert("validate_".to_string(), 50.0); // 50MB
        memory_multipliers.insert("backup_".to_string(), 100.0); // 100MB
        memory_multipliers.insert("collect_data".to_string(), 300.0); // 300MB

        let mut disk_multipliers = HashMap::new();
        disk_multipliers.insert("file_operation".to_string(), 1000.0); // 1000 ops/sec
        disk_multipliers.insert("backup_".to_string(), 500.0); // 500 ops/sec
        disk_multipliers.insert("apply_config".to_string(), 100.0); // 100 ops/sec

        let mut network_multipliers = HashMap::new();
        network_multipliers.insert("collect_data".to_string(), 1024.0 * 1024.0); // 1MB/s
        network_multipliers.insert("monitoring".to_string(), 512.0 * 1024.0); // 512KB/s

        Self {
            cpu_multipliers,
            memory_multipliers,
            disk_multipliers,
            network_multipliers,
        }
    }
}

/// Historical resource usage profile
#[derive(Debug, Clone)]
pub struct ResourceProfile {
    /// Command pattern this profile applies to
    pub command_pattern: String,
    /// Average CPU usage
    pub avg_cpu_usage: f32,
    /// Peak memory usage in MB
    pub peak_memory_mb: f32,
    /// Average disk I/O operations per second
    pub avg_disk_ops_per_sec: f32,
    /// Average network bandwidth in bytes per second
    pub avg_network_bps: f32,
    /// Number of samples this profile is based on
    pub sample_count: u32,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// System resource limits
#[derive(Debug, Clone)]
pub struct SystemResourceLimits {
    /// Maximum CPU cores available
    pub max_cpu_cores: u32,
    /// Maximum memory in bytes
    pub max_memory_bytes: u64,
    /// Maximum disk I/O operations per second
    pub max_disk_ops_per_sec: u32,
    /// Maximum network bandwidth in bytes per second
    pub max_network_bps: u64,
}

impl Default for SystemResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_cores: 8,
            max_memory_bytes: 16 * 1024 * 1024 * 1024, // 16GB
            max_disk_ops_per_sec: 10000,
            max_network_bps: 1024 * 1024 * 1024, // 1GB/s
        }
    }
}

impl ResourceEstimator {
    /// Create a new resource estimator
    pub fn new() -> Self {
        Self {
            config: ResourceEstimationConfig::default(),
            historical_data: HashMap::new(),
            system_limits: SystemResourceLimits::default(),
        }
    }

    /// Create estimator with custom configuration
    pub fn with_config(config: ResourceEstimationConfig) -> Self {
        let mut estimator = Self::new();
        estimator.config = config;
        estimator
    }

    /// Estimate resource requirements for a plan
    pub async fn estimate_plan_resources(&self, steps: &[PlanStep]) -> Result<ResourceRequirements, PlanningError> {
        debug!("Estimating resources for {} steps", steps.len());

        let mut total_cpu = 0.0;
        let mut total_memory = 0;
        let mut total_disk_space = 0;
        let mut total_network_bandwidth = 0;
        let mut exclusive_resources = Vec::new();

        for step in steps {
            let step_resources = self.estimate_step_resources(step).await?;
            
            total_cpu += step_resources.cpu_usage;
            total_memory += step_resources.memory_usage;
            total_disk_space += step_resources.disk_space;
            total_network_bandwidth += step_resources.network_bandwidth;
            
            exclusive_resources.extend(step_resources.exclusive_resources);
        }

        // Apply safety margin
        total_cpu *= 1.0 + self.config.safety_margin;
        total_memory = (total_memory as f64 * (1.0 + self.config.safety_margin as f64)) as u64;
        total_disk_space = (total_disk_space as f64 * (1.0 + self.config.safety_margin as f64)) as u64;
        total_network_bandwidth = (total_network_bandwidth as f64 * (1.0 + self.config.safety_margin as f64)) as u64;

        // Ensure we don't exceed system limits
        total_cpu = total_cpu.min(1.0);
        total_memory = total_memory.min(self.system_limits.max_memory_bytes);

        // Remove duplicate exclusive resources
        exclusive_resources.sort();
        exclusive_resources.dedup();

        Ok(ResourceRequirements {
            cpu_usage: total_cpu,
            memory_usage: total_memory,
            disk_space: total_disk_space,
            network_bandwidth: total_network_bandwidth,
            exclusive_resources,
        })
    }

    /// Estimate CPU usage for steps
    pub async fn estimate_cpu_usage(&self, steps: &[PlanStep]) -> Result<CpuUsage, PlanningError> {
        let mut total_percentage = 0.0;
        let mut max_cores_needed = 1;

        for step in steps {
            let step_cpu = self.estimate_step_cpu_usage(step).await?;
            total_percentage += step_cpu.percentage;
            max_cores_needed = max_cores_needed.max(step_cpu.cores_needed);
        }

        // Apply safety margin and parallel execution considerations
        if self.config.consider_parallel_execution {
            // Assume some steps can run in parallel, reduce total CPU estimate
            total_percentage *= 0.7; // 30% reduction for parallel execution
        }

        total_percentage *= 1.0 + self.config.safety_margin;
        total_percentage = total_percentage.min(1.0);

        Ok(CpuUsage {
            percentage: total_percentage,
            cores_needed: max_cores_needed.min(self.system_limits.max_cpu_cores),
        })
    }

    /// Estimate memory usage for steps
    pub async fn estimate_memory_usage(&self, steps: &[PlanStep]) -> Result<MemoryUsage, PlanningError> {
        let mut peak_bytes = 0;
        let mut total_average = 0;

        for step in steps {
            let step_memory = self.estimate_step_memory_usage(step).await?;
            peak_bytes = peak_bytes.max(step_memory.peak_bytes);
            total_average += step_memory.average_bytes;
        }

        // Apply safety margin
        peak_bytes = (peak_bytes as f64 * (1.0 + self.config.safety_margin as f64)) as u64;
        total_average = (total_average as f64 * (1.0 + self.config.safety_margin as f64)) as u64;

        // Ensure we don't exceed system limits
        peak_bytes = peak_bytes.min(self.system_limits.max_memory_bytes);
        total_average = total_average.min(self.system_limits.max_memory_bytes);

        Ok(MemoryUsage {
            peak_bytes,
            average_bytes: total_average / steps.len() as u64,
        })
    }

    /// Estimate disk I/O for steps
    pub async fn estimate_disk_io(&self, steps: &[PlanStep]) -> Result<DiskIO, PlanningError> {
        let mut total_read_ops = 0;
        let mut total_write_ops = 0;
        let mut total_read_bytes = 0;
        let mut total_write_bytes = 0;

        for step in steps {
            let step_disk_io = self.estimate_step_disk_io(step).await?;
            total_read_ops += step_disk_io.read_ops_per_sec;
            total_write_ops += step_disk_io.write_ops_per_sec;
            total_read_bytes += step_disk_io.total_read_bytes;
            total_write_bytes += step_disk_io.total_write_bytes;
        }

        // Apply safety margin
        total_read_ops = (total_read_ops as f64 * (1.0 + self.config.safety_margin as f64)) as u32;
        total_write_ops = (total_write_ops as f64 * (1.0 + self.config.safety_margin as f64)) as u32;
        total_read_bytes = (total_read_bytes as f64 * (1.0 + self.config.safety_margin as f64)) as u64;
        total_write_bytes = (total_write_bytes as f64 * (1.0 + self.config.safety_margin as f64)) as u64;

        Ok(DiskIO {
            read_ops_per_sec: total_read_ops.min(self.system_limits.max_disk_ops_per_sec),
            write_ops_per_sec: total_write_ops.min(self.system_limits.max_disk_ops_per_sec),
            total_read_bytes,
            total_write_bytes,
        })
    }

    /// Estimate network I/O for steps
    pub async fn estimate_network_io(&self, steps: &[PlanStep]) -> Result<NetworkIO, PlanningError> {
        let mut incoming_bps = 0;
        let mut outgoing_bps = 0;
        let mut total_bytes = 0;

        for step in steps {
            let step_network_io = self.estimate_step_network_io(step).await?;
            incoming_bps += step_network_io.incoming_bps;
            outgoing_bps += step_network_io.outgoing_bps;
            total_bytes += step_network_io.total_bytes;
        }

        // Apply safety margin
        incoming_bps = (incoming_bps as f64 * (1.0 + self.config.safety_margin as f64)) as u64;
        outgoing_bps = (outgoing_bps as f64 * (1.0 + self.config.safety_margin as f64)) as u64;
        total_bytes = (total_bytes as f64 * (1.0 + self.config.safety_margin as f64)) as u64;

        Ok(NetworkIO {
            incoming_bps: incoming_bps.min(self.system_limits.max_network_bps),
            outgoing_bps: outgoing_bps.min(self.system_limits.max_network_bps),
            total_bytes,
        })
    }

    /// Validate resource requirements against system limits
    pub async fn validate_resources(&self, requirements: &ResourceRequirements) -> Result<(), String> {
        debug!("Validating resource requirements");

        // Check CPU usage
        if requirements.cpu_usage > 1.0 {
            return Err(format!("CPU usage exceeds 100%: {:.1}%", requirements.cpu_usage * 100.0));
        }

        // Check memory usage
        if requirements.memory_usage > self.system_limits.max_memory_bytes {
            return Err(format!(
                "Memory usage exceeds system limit: {} > {} bytes",
                requirements.memory_usage,
                self.system_limits.max_memory_bytes
            ));
        }

        // Check network bandwidth
        if requirements.network_bandwidth > self.system_limits.max_network_bps {
            return Err(format!(
                "Network bandwidth exceeds system limit: {} > {} bps",
                requirements.network_bandwidth,
                self.system_limits.max_network_bps
            ));
        }

        Ok(())
    }

    /// Estimate resources for a single step
    async fn estimate_step_resources(&self, step: &PlanStep) -> Result<ResourceRequirements, PlanningError> {
        // Try to use historical data first
        if self.config.use_historical_data {
            if let Some(profile) = self.find_matching_profile(&step.command) {
                return Ok(self.profile_to_requirements(profile, step));
            }
        }

        // Fall back to heuristic estimation
        self.estimate_step_resources_heuristic(step).await
    }

    /// Find matching historical profile for a command
    fn find_matching_profile(&self, command: &str) -> Option<&ResourceProfile> {
        // Try exact match first
        if let Some(profile) = self.historical_data.get(command) {
            return Some(profile);
        }

        // Try pattern matching
        for (pattern, profile) in &self.historical_data {
            if command.starts_with(pattern) || pattern.contains("*") {
                return Some(profile);
            }
        }

        None
    }

    /// Convert resource profile to requirements
    fn profile_to_requirements(&self, profile: &ResourceProfile, step: &PlanStep) -> ResourceRequirements {
        let duration_secs = step.timeout.as_secs();
        
        ResourceRequirements {
            cpu_usage: profile.avg_cpu_usage,
            memory_usage: (profile.peak_memory_mb * 1024.0 * 1024.0) as u64,
            disk_space: (profile.avg_disk_ops_per_sec as u64 * duration_secs * 4096), // Assume 4KB per op
            network_bandwidth: profile.avg_network_bps as u64,
            exclusive_resources: self.determine_exclusive_resources(step),
        }
    }

    /// Estimate step resources using heuristics
    async fn estimate_step_resources_heuristic(&self, step: &PlanStep) -> Result<ResourceRequirements, PlanningError> {
        let cpu_usage = self.get_cpu_multiplier(&step.command);
        let memory_usage = (self.get_memory_multiplier(&step.command) * 1024.0 * 1024.0) as u64; // Convert MB to bytes
        let disk_space = self.estimate_disk_space_heuristic(step);
        let network_bandwidth = self.get_network_multiplier(&step.command) as u64;
        let exclusive_resources = self.determine_exclusive_resources(step);

        Ok(ResourceRequirements {
            cpu_usage,
            memory_usage,
            disk_space,
            network_bandwidth,
            exclusive_resources,
        })
    }

    /// Get CPU multiplier for command type
    fn get_cpu_multiplier(&self, command: &str) -> f32 {
        for (pattern, multiplier) in &self.config.default_multipliers.cpu_multipliers {
            if command.starts_with(pattern) {
                return *multiplier;
            }
        }
        0.1 // Default low CPU usage
    }

    /// Get memory multiplier for command type
    fn get_memory_multiplier(&self, command: &str) -> f32 {
        for (pattern, multiplier) in &self.config.default_multipliers.memory_multipliers {
            if command.starts_with(pattern) {
                return *multiplier;
            }
        }
        100.0 // Default 100MB
    }

    /// Get network multiplier for command type
    fn get_network_multiplier(&self, command: &str) -> f32 {
        for (pattern, multiplier) in &self.config.default_multipliers.network_multipliers {
            if command.starts_with(pattern) {
                return *multiplier;
            }
        }
        0.0 // Default no network usage
    }

    /// Estimate disk space usage heuristically
    fn estimate_disk_space_heuristic(&self, step: &PlanStep) -> u64 {
        match step.command.as_str() {
            cmd if cmd.contains("backup") => 1024 * 1024 * 1024, // 1GB for backups
            cmd if cmd.contains("file_operation") => 512 * 1024 * 1024, // 512MB for file ops
            cmd if cmd.contains("collect_data") => 256 * 1024 * 1024, // 256MB for data collection
            _ => 10 * 1024 * 1024, // 10MB default
        }
    }

    /// Determine exclusive resources needed by a step
    fn determine_exclusive_resources(&self, step: &PlanStep) -> Vec<String> {
        let mut resources = Vec::new();

        match step.command.as_str() {
            cmd if cmd.contains("config") => {
                resources.push("system_configuration".to_string());
            },
            cmd if cmd.contains("process") => {
                resources.push("process_management".to_string());
            },
            cmd if cmd.contains("file_operation") => {
                // Extract file paths from parameters if available
                if let Some(path) = step.parameters.get("path") {
                    if let Ok(path_str) = serde_json::from_value::<String>(path.clone()) {
                        resources.push(format!("file_lock:{}", path_str));
                    }
                }
            },
            _ => {}
        }

        resources
    }

    /// Estimate CPU usage for a single step
    async fn estimate_step_cpu_usage(&self, step: &PlanStep) -> Result<CpuUsage, PlanningError> {
        let percentage = self.get_cpu_multiplier(&step.command);
        let cores_needed = if percentage > 0.5 { 2 } else { 1 };

        Ok(CpuUsage {
            percentage,
            cores_needed,
        })
    }

    /// Estimate memory usage for a single step
    async fn estimate_step_memory_usage(&self, step: &PlanStep) -> Result<MemoryUsage, PlanningError> {
        let memory_mb = self.get_memory_multiplier(&step.command);
        let memory_bytes = (memory_mb * 1024.0 * 1024.0) as u64;

        Ok(MemoryUsage {
            peak_bytes: memory_bytes,
            average_bytes: (memory_bytes as f64 * 0.7) as u64, // Assume 70% average usage
        })
    }

    /// Estimate disk I/O for a single step
    async fn estimate_step_disk_io(&self, step: &PlanStep) -> Result<DiskIO, PlanningError> {
        let base_ops = match step.command.as_str() {
            cmd if cmd.contains("file") => 1000,
            cmd if cmd.contains("backup") => 500,
            cmd if cmd.contains("config") => 100,
            _ => 50,
        };

        let duration_secs = step.timeout.as_secs();
        let total_ops = base_ops * duration_secs as u32;

        Ok(DiskIO {
            read_ops_per_sec: base_ops / 2,
            write_ops_per_sec: base_ops / 2,
            total_read_bytes: (total_ops as u64 / 2) * 4096, // Assume 4KB per op
            total_write_bytes: (total_ops as u64 / 2) * 4096,
        })
    }

    /// Estimate network I/O for a single step
    async fn estimate_step_network_io(&self, step: &PlanStep) -> Result<NetworkIO, PlanningError> {
        let bandwidth = self.get_network_multiplier(&step.command) as u64;
        let duration_secs = step.timeout.as_secs();
        let total_bytes = bandwidth * duration_secs;

        Ok(NetworkIO {
            incoming_bps: bandwidth / 2,
            outgoing_bps: bandwidth / 2,
            total_bytes,
        })
    }

    /// Update historical data with actual resource usage
    pub fn update_historical_data(&mut self, command: &str, actual_usage: ResourceProfile) {
        self.historical_data.insert(command.to_string(), actual_usage);
    }

    /// Get system resource limits
    pub fn get_system_limits(&self) -> &SystemResourceLimits {
        &self.system_limits
    }

    /// Update system resource limits
    pub fn update_system_limits(&mut self, limits: SystemResourceLimits) {
        self.system_limits = limits;
    }
}

impl Default for ResourceEstimator {
    fn default() -> Self {
        Self::new()
    }
}