//! Transformation & rollback correctness tests.
//! Guarantees: rollback fails closed, snapshot lifecycle enforced.

#[cfg(test)]
mod rollback_tests {
    use crate::arh::rollback_manager::{InMemoryWorkspaceIO, RollbackManager, RollbackScope};
    use crate::arh::rollback_manager::WorkspaceIO;
    use std::rc::Rc;

    struct SharedIO {
        inner: Rc<InMemoryWorkspaceIO>,
    }

    impl WorkspaceIO for SharedIO {
        fn read_file(&self, path: &str) -> Result<String, String> {
            self.inner.read_file(path)
        }

        fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
            self.inner.write_file(path, content)
        }

        fn reset_security_state(&self) -> Result<(), String> {
            self.inner.reset_security_state()
        }
    }

    #[test]
    fn rollback_fails_without_snapshot() {
        let io = Box::new(InMemoryWorkspaceIO::new());
        let manager = RollbackManager::new(io);
        let scope = RollbackScope {
            module_id: "m".to_string(),
            snapshot_id: "missing".to_string(),
        };
        let result = manager.rollback(scope);
        assert!(result.is_err(), "rollback must fail if snapshot missing");
    }

    #[test]
    fn rollback_restores_byte_identical_state() {
        let shared = Rc::new(InMemoryWorkspaceIO::new());
        shared.write_file("file.rs", "before").unwrap();

        let manager = RollbackManager::new(Box::new(SharedIO { inner: shared.clone() }));
        let scope = manager
            .begin_scope("mod", &["file.rs".to_string()])
            .unwrap();

        shared.write_file("file.rs", "after").unwrap();
        manager.rollback(scope).unwrap();

        let restored = shared.read_file("file.rs").unwrap();
        assert_eq!(restored, "before", "rollback must restore byte-identical state");
    }

    #[test]
    fn rollback_is_one_shot_snapshot_consumed() {
        let shared = Rc::new(InMemoryWorkspaceIO::new());
        shared.write_file("file.rs", "before").unwrap();

        let manager = RollbackManager::new(Box::new(SharedIO { inner: shared.clone() }));
        let scope = manager
            .begin_scope("mod", &["file.rs".to_string()])
            .unwrap();

        shared.write_file("file.rs", "after").unwrap();
        manager.rollback(scope.clone()).unwrap();

        let second = manager.rollback(scope);
        assert!(second.is_err(), "rollback must fail after snapshot consumed");
    }

    #[test]
    fn commit_consumes_snapshot_and_prevents_rollback() {
        let shared = Rc::new(InMemoryWorkspaceIO::new());
        shared.write_file("file.rs", "before").unwrap();

        let manager = RollbackManager::new(Box::new(SharedIO { inner: shared.clone() }));
        let scope = manager
            .begin_scope("mod", &["file.rs".to_string()])
            .unwrap();

        manager.commit(&scope.snapshot_id).unwrap();
        let result = manager.rollback(scope);
        assert!(result.is_err(), "rollback must fail after commit");
    }
}

#[cfg(test)]
mod engine_validation_tests {
    use crate::arh::application_validation::{ApplicationValidation, ValidationResult};
    use crate::arh::fix_application_engine::{
        ApprovalArtifact, ApprovalDecision, ApprovalMode, ApplyOutcome, FixApplicationEngine,
        FixPlan,
    };
    use crate::arh::rollback_manager::{InMemoryWorkspaceIO, RollbackManager};
    use crate::arh::transformation_system::{TransformationPlan, TransformationSystem};

    #[test]
    fn kernel_fix_requires_opt_in_approval() {
        let engine = FixApplicationEngine::new(
            ApplicationValidation::new(),
            TransformationSystem::new(),
            RollbackManager::new(Box::new(InMemoryWorkspaceIO::new())),
        );

        let plan = FixPlan {
            violation_ids: vec!["v".to_string()],
            module_id: "m".to_string(),
            plans: vec![TransformationPlan {
                module_id: "m".to_string(),
                file: "f.rs".to_string(),
                range: "1..2".to_string(),
                summary: "s".to_string(),
                dry_run: true,
            }],
            approval: ApprovalArtifact {
                mode: ApprovalMode::Preview,
                decision: ApprovalDecision::Approved,
                approver_id: "user".to_string(),
                timestamp: "2026-02-01T00:00:00Z".to_string(),
                proof: "p".to_string(),
                kernel_opt_in: false,
            },
            is_kernel: true,
        };

        let result = engine.apply_plan(plan);
        assert_eq!(result.outcome, ApplyOutcome::Failed);
        assert!(
            result.message.contains("Kernel fix requires explicit opt-in approval"),
            "kernel approval gate must fail closed"
        );
    }

    #[test]
    fn cross_module_plan_is_rejected() {
        let validator = ApplicationValidation::new();
        let plan = FixPlan {
            violation_ids: vec!["v".to_string()],
            module_id: "m1".to_string(),
            plans: vec![TransformationPlan {
                module_id: "m2".to_string(),
                file: "f.rs".to_string(),
                range: "1..2".to_string(),
                summary: "s".to_string(),
                dry_run: true,
            }],
            approval: ApprovalArtifact {
                mode: ApprovalMode::Safe,
                decision: ApprovalDecision::Approved,
                approver_id: "user".to_string(),
                timestamp: "2026-02-01T00:00:00Z".to_string(),
                proof: "p".to_string(),
                kernel_opt_in: false,
            },
            is_kernel: false,
        };

        let report = validator.pre_apply(&plan);
        assert_eq!(report.result, ValidationResult::Failed);
        assert!(
            report.message.contains("Cross-module plan is forbidden"),
            "cross-module plan must be rejected"
        );
    }
}
