import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
  Index,
  ManyToOne,
  JoinColumn,
} from 'typeorm';
import { StepState } from '../types/step-state.enum';
import { Workflow } from './workflow.entity';

@Entity('workflow_steps')
@Index(['workflowId'])
@Index(['state'])
export class WorkflowStep {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column()
  workflowId: string;

  @Column()
  stepName: string;

  @Column()
  stepIndex: number;

  @Column({
    type: 'simple-enum',
    enum: StepState,
    default: StepState.PENDING,
  })
  state: StepState;

  @Column({ type: 'simple-json', nullable: true })
  input?: Record<string, any>;

  @Column({ type: 'simple-json', nullable: true })
  output?: Record<string, any>;

  @Column({ type: 'simple-json', nullable: true })
  config?: Record<string, any>;

  @Column({ default: 0 })
  retryCount: number;

  @Column({ default: 3 })
  maxRetries: number;

  @Column({ nullable: true })
  startedAt?: Date;

  @Column({ nullable: true })
  completedAt?: Date;

  @Column({ nullable: true })
  failedAt?: Date;

  @Column({ nullable: true })
  failureReason?: string;

  @Column({ nullable: true })
  nextRetryAt?: Date;

  @Column({ default: false })
  requiresCompensation: boolean;

  @Column({ default: false })
  isCompensated: boolean;

  @Column({ nullable: true })
  compensatedAt?: Date;

  @Column({ nullable: true })
  compensationStepName?: string;

  @Column({ type: 'simple-json', nullable: true })
  compensationConfig?: Record<string, any>;

  @Column({ default: false })
  isIdempotent: boolean;

  @Column({ nullable: true })
  idempotencyKey?: string;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  @ManyToOne(() => Workflow, (workflow) => workflow.steps, {
    onDelete: 'CASCADE',
  })
  @JoinColumn({ name: 'workflowId' })
  workflow: Workflow;
}
