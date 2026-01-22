import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ConfigModule } from '@nestjs/config';
import { Workflow } from './entities/workflow.entity';
import { WorkflowStep } from './entities/workflow-step.entity';
import { WorkflowExecutionService } from './services/workflow-execution.service';
import { WorkflowStateMachineService } from './services/workflow-state-machine.service';
import { IdempotencyService } from './services/idempotency.service';
import { WorkflowService } from './services/workflow.service';
import { WorkflowAdminController } from './controllers/workflow-admin.controller';

@Module({
  imports: [
    ConfigModule,
    TypeOrmModule.forFeature([Workflow, WorkflowStep]),
  ],
  controllers: [WorkflowAdminController],
  providers: [
    WorkflowExecutionService,
    WorkflowStateMachineService,
    IdempotencyService,
    WorkflowService,
  ],
  exports: [
    WorkflowExecutionService,
    WorkflowStateMachineService,
    IdempotencyService,
    WorkflowService,
  ],
})
export class WorkflowModule {}
