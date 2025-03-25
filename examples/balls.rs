use clod::{
    style::{CanvasAlignment, Circle, CircleLike, Stylize},
    App, AppResult,
};

use glam::Vec2;
use rand::{thread_rng, Rng};
use rapier2d::{
    na::Vector2,
    prelude::{
        CCDSolver, ColliderBuilder, ColliderHandle, ColliderSet, DefaultBroadPhase,
        ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase,
        PhysicsPipeline, QueryPipeline, RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
    },
};

struct Entity {
    handle: RigidBodyHandle,
}

#[derive(Default)]
struct MyApp {
    entities: Vec<Entity>,
    boundaries: Vec<ColliderHandle>,
    rigidbody_set: RigidBodySet,
    collider_set: ColliderSet,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
}

impl App for MyApp {
    fn update(&mut self, state: &mut clod::State) -> Result<(), String> {
        let gravity = Vector2::new(0.0, 0.0);
        let integration_parameters = IntegrationParameters {
            dt: state.delta_seconds(),
            ..IntegrationParameters::default()
        };

        self.physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigidbody_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );

        for entity in self.entities.iter() {
            let ball_body = &mut self.rigidbody_set[entity.handle];
            for collider_handle in ball_body.colliders() {
                let ball_collider = &self.collider_set[*collider_handle];
                if let Some(ball) = ball_collider.shape().as_ball() {
                    state.aa_circle(
                        Vec2::new(ball_body.translation().x, ball_body.translation().y),
                        Circle::with_radius(ball.radius).solid(),
                    );
                }
            }
        }

        state.print(
            "Press A to spawn entity"
                .align(CanvasAlignment::TOP | CanvasAlignment::LEFT)
                .padding(2),
        );

        Ok(())
    }

    fn on_key_event(&mut self, state: &mut clod::State, event: crossterm::event::KeyEvent) {
        let bounds = state.canvas_size().as_vec2();
        if let crossterm::event::KeyCode::Char('a') = event.code {
            /* Create the bouncing ball. */
            let rigid_body = RigidBodyBuilder::dynamic()
                .translation(Vector2::new(bounds.x / 2.0, bounds.y / 2.0))
                .build();
            let collider = ColliderBuilder::ball(thread_rng().gen_range(1.0..5.0))
                .restitution(1.0)
                .friction(0.0)
                .build();
            let ball_body_handle = self.rigidbody_set.insert(rigid_body);
            let _ = self.collider_set.insert_with_parent(
                collider,
                ball_body_handle,
                &mut self.rigidbody_set,
            );
            self.rigidbody_set[ball_body_handle].apply_impulse(Vector2::new(1000.0, 1000.0), true);

            self.entities.push(Entity {
                handle: ball_body_handle,
            });
        }
    }

    fn init(&mut self, _state: &mut clod::State) -> Result<(), String> {
        let bounds = _state.canvas_size().as_vec2();

        let bottom = ColliderBuilder::cuboid(bounds.x, 5.1)
            .position(Vector2::new(bounds.x / 2.0, 0.0).into())
            .friction(0.0)
            .build();
        let top = ColliderBuilder::cuboid(bounds.x, 5.1)
            .position(Vector2::new(bounds.x / 2.0, bounds.y).into())
            .friction(0.0)
            .build();
        let left = ColliderBuilder::cuboid(5.1, bounds.y)
            .position(Vector2::new(0.0, bounds.y / 2.0).into())
            .friction(0.0)
            .build();
        let right = ColliderBuilder::cuboid(5.1, bounds.y)
            .position(Vector2::new(bounds.x, bounds.y / 2.0).into())
            .friction(0.0)
            .build();

        self.boundaries.push(self.collider_set.insert(bottom));
        self.boundaries.push(self.collider_set.insert(top));
        self.boundaries.push(self.collider_set.insert(left));
        self.boundaries.push(self.collider_set.insert(right));

        Ok(())
    }
}

fn main() -> AppResult {
    let mut app = MyApp::default();
    app.run()
}

#[cfg(test)]
mod tests {}
