use rapier3d::prelude::*;
use nalgebra::Vector3;

pub struct PhysicsWorld {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    gravity: Vector3<f32>,
    integration_parameters: IntegrationParameters,
    
    // Track our agent
    pub agent_handle: Option<RigidBodyHandle>,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: IntegrationParameters::default(),
            agent_handle: None,
        }
    }

    /// Initialize the world with a terrain heightmap
    pub fn init_terrain(&mut self, heightmap: &[f32], width: usize, height: usize) {
        // Create HeightField collider
        // Rapier expects DMatrix or similar for heightfield
        // We will construct it manually
        
        // Scale: 1 unit = 1 meter
        let scale = Vector3::new(width as f32, 50.0, height as f32);
        
        // Convert flat Vec to DMatrix (nalgebra)
        let heights = nalgebra::DMatrix::from_iterator(height, width, heightmap.iter().cloned());
        
        let collider = ColliderBuilder::heightfield(heights, scale)
            .translation(vector![0.0, 0.0, 0.0])
            .build();
            
        self.collider_set.insert(collider);
        
        // Spawn Agent above the terrain
        self.spawn_agent(width as f32 / 2.0, height as f32 / 2.0);
    }

    fn spawn_agent(&mut self, x: f32, z: f32) {
        // Create a dynamic rigid body (Agent)
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![x, 50.0, z]) // Start high up
            .linear_damping(0.5)
            .angular_damping(0.5)
            .additional_mass(70.0) // 70kg agent
            .build();
            
        let collider = ColliderBuilder::capsule_y(0.9, 0.3) // 1.8m tall, 0.6m wide
            .restitution(0.0) // No bounce
            .friction(0.7)
            .build();
            
        let handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set.insert_with_parent(collider, handle, &mut self.rigid_body_set);
        
        self.agent_handle = Some(handle);
    }

    /// Step the simulation forward
    /// Returns the agent's position as string for debugging
    pub fn step(&mut self, impulse: Option<Vector3<f32>>) -> String {
        // Apply impulse if provided
        if let Some(force) = impulse {
            if let Some(handle) = self.agent_handle {
                let body = &mut self.rigid_body_set[handle];
                body.apply_impulse(force, true);
            }
        }
    
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );
        
        // Get agent state
        if let Some(handle) = self.agent_handle {
            let body = &self.rigid_body_set[handle];
            let pos = body.translation();
            format!("Agent Pos: {:.2}, {:.2}, {:.2}", pos.x, pos.y, pos.z)
        } else {
            "Agent Missing".to_string()
        }
    }
}
