import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { In, LessThan, Repository } from 'typeorm';
import { Workflow } from '../entities/workflow.entity';
import { WorkflowStep } from '../entities/workflow-step.entity';
import { WorkflowState } from '../types/workflow-state.enum';
import { StepState } from '../types/step-state.enum';
import { WorkflowExecutionService } from './workflow-execution.service';
import { contractDeploymentWorkflow } from '../examples/contract-deployment.workflow';
import { tradeExecutionWorkflow } from '../examples/trade-execution.workflow';

@Injectable()
export class WorkflowService implements OnModuleInit {
  private readonly logger = new Logger(WorkflowService.name);

  constructor(
    @InjectRepository(Workflow)
    private readonly workflowRepository: Repository<Workflow>,
    @InjectRepository(WorkflowStep)
    private readonly stepRepository: Repository<WorkflowStep>,
    private readonly workflowExecutionService: WorkflowExecutionService,
  ) {}

  async onModuleInit() {
    // Register built-in workflow definitions
    this.workflowExecutionService.registerWorkflowDefinition(contractDeploymentWorkflow);
    this.workflowExecutionService.registerWorkflowDefinition(tradeExecutionWorkflow);
    
    this.logger.log('WorkflowService initialized with built-in workflow definitions');
  }

  /**
   * Start a new workflow
   */
  async startWorkflow(
    type: string,
    input: Record<string, any>,
    userId?: string,
    walletAddress?: string,
    context?: Record<string, any>,
  ): Promise<Workflow> {
    this.logger.log(`Starting workflow of type: ${type} for user: ${userId}`);
    
    return await this.workflowExecutionService.startWorkflow(
      type,
      input,
      userId,
      walletAddress,
      context,
    );
  }

  /**
   * Get workflow by ID
   */
  async getWorkflow(id: string): Promise<Workflow | null> {
    return await this.workflowRepository.findOne({
      where: { id },
      relations: ['steps'],
    });
  }

  /**
   * Get workflows by user
   */
  async getUserWorkflows(
    userId: string,
    page: number = 1,
    limit: number = 20,
  ): Promise<{ workflows: Workflow[]; total: number }> {
    const skip = (page - 1) * limit;
    
    const [workflows, total] = await this.workflowRepository.findAndCount({
      where: { userId },
      relations: ['steps'],
      order: { createdAt: 'DESC' },
      skip,
      take: limit,
    });

    return { workflows, total };
  }

  /**
   * Get workflows by wallet address
   */
  async getWalletWorkflows(
    walletAddress: string,
    page: number = 1,
    limit: number = 20,
  ): Promise<{ workflows: Workflow[]; total: number }> {
    const skip = (page - 1) * limit;
    
    const [workflows, total] = await this.workflowRepository.findAndCount({
      where: { walletAddress },
      relations: ['steps'],
      order: { createdAt: 'DESC' },
      skip,
      take: limit,
    });

    return { workflows, total };
  }

  /**
   * Get workflows by state
   */
  async getWorkflowsByState(
    state: WorkflowState,
    page: number = 1,
    limit: number = 20,
  ): Promise<{ workflows: Workflow[]; total: number }> {
    const skip = (page - 1) * limit;
    
    const [workflows, total] = await this.workflowRepository.findAndCount({
      where: { state },
      relations: ['steps'],
      order: { createdAt: 'DESC' },
      skip,
      take: limit,
    });

    return { workflows, total };
  }

  /**
   * Get failed workflows that can be retried
   */
  async getRetryableWorkflows(): Promise<Workflow[]> {
    return await this.workflowRepository.find({
      where: { 
        state: WorkflowState.FAILED,
        retryCount: 0, // Only show workflows that haven't been retried
      },
      relations: ['steps'],
      order: { failedAt: 'DESC' },
    });
  }

  /**
   * Get workflows that need compensation
   */
  async getCompensableWorkflows(): Promise<Workflow[]> {
    return await this.workflowRepository.find({
      where: { 
        requiresCompensation: true,
        isCompensated: false,
        state: In([WorkflowState.COMPLETED, WorkflowState.FAILED, WorkflowState.CANCELLED]),
      },
      relations: ['steps'],
      order: { createdAt: 'DESC' },
    });
  }

  /**
   * Cancel a workflow
   */
  async cancelWorkflow(id: string): Promise<void> {
    await this.workflowExecutionService.cancelWorkflow(id);
  }

  /**
   * Retry a failed workflow
   */
  async retryWorkflow(id: string): Promise<void> {
    await this.workflowExecutionService.retryWorkflow(id);
  }

  /**
   * Compensate a workflow
   */
  async compensateWorkflow(id: string): Promise<void> {
    await this.workflowExecutionService.compensateWorkflow(id);
  }

  /**
   * Get workflow statistics
   */
  async getWorkflowStats(): Promise<any> {
    const stats = await this.workflowRepository
      .createQueryBuilder('workflow')
      .select('workflow.state', 'state')
      .addSelect('COUNT(*)', 'count')
      .groupBy('workflow.state')
      .getRawMany();

    const totalWorkflows = stats.reduce((sum, stat) => sum + parseInt(stat.count), 0);
    const stateStats = stats.reduce((acc, stat) => {
      acc[stat.state] = parseInt(stat.count);
      return acc;
    }, {});

    // Get step stats
    const stepStats = await this.stepRepository
      .createQueryBuilder('step')
      .select('step.state', 'state')
      .addSelect('COUNT(*)', 'count')
      .groupBy('step.state')
      .getRawMany();

    const totalSteps = stepStats.reduce((sum, stat) => sum + parseInt(stat.count), 0);
    const stepStateStats = stepStats.reduce((acc, stat) => {
      acc[stat.state] = parseInt(stat.count);
      return acc;
    }, {});

    return {
      workflows: {
        total: totalWorkflows,
        byState: stateStats,
      },
      steps: {
        total: totalSteps,
        byState: stepStateStats,
      },
    };
  }

  /**
   * Search workflows by idempotency key
   */
  async searchByIdempotencyKey(idempotencyKey: string): Promise<Workflow | null> {
    return await this.workflowRepository.findOne({
      where: { idempotencyKey },
      relations: ['steps'],
    });
  }

  /**
   * Get workflow execution summary
   */
  async getWorkflowExecutionSummary(id: string): Promise<any> {
    const workflow = await this.getWorkflow(id);
    
    if (!workflow) {
      throw new Error('Workflow not found');
    }

    const completedSteps = workflow.steps.filter(step => 
      [StepState.COMPLETED, StepState.SKIPPED].includes(step.state)
    ).length;
    
    const failedSteps = workflow.steps.filter(step => step.state === StepState.FAILED).length;
    const runningSteps = workflow.steps.filter(step => step.state === StepState.RUNNING).length;
    
    const totalExecutionTime = workflow.completedAt && workflow.startedAt
      ? workflow.completedAt.getTime() - workflow.startedAt.getTime()
      : null;

    return {
      workflowId: workflow.id,
      type: workflow.type,
      state: workflow.state,
      progress: {
        totalSteps: workflow.totalSteps,
        completedSteps,
        failedSteps,
        runningSteps,
        currentStep: workflow.currentStepIndex,
        completionPercentage: Math.round((completedSteps / workflow.totalSteps) * 100),
      },
      timing: {
        createdAt: workflow.createdAt,
        startedAt: workflow.startedAt,
        completedAt: workflow.completedAt,
        totalExecutionTime,
        averageStepTime: totalExecutionTime && completedSteps > 0
          ? totalExecutionTime / completedSteps
          : null,
      },
      retries: {
        workflowRetries: workflow.retryCount,
        maxRetries: workflow.maxRetries,
        stepRetries: workflow.steps.reduce((sum, step) => sum + step.retryCount, 0),
      },
    };
  }

  /**
   * Clean up old completed workflows
   */
  async cleanupOldWorkflows(daysOld: number = 30): Promise<number> {
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - daysOld);

    const result = await this.workflowRepository.delete({
      state: In([WorkflowState.COMPLETED, WorkflowState.COMPENSATED]),
      completedAt: LessThan(cutoffDate),
    });

    this.logger.log(`Cleaned up ${result.affected} old workflows`);
    return result.affected || 0;
  }

  /**
   * Get workflows that need retry (failed and within retry window)
   */
  async getWorkflowsNeedingRetry(): Promise<Workflow[]> {
    const now = new Date();
    
    return await this.workflowRepository.find({
      where: {
        state: WorkflowState.FAILED,
        retryCount: 0, // Only workflows that haven't been retried yet
        nextRetryAt: LessThan(now),
      },
      relations: ['steps'],
      order: { nextRetryAt: 'ASC' },
    });
  }

  /**
   * Process workflows that need retry
   */
  async processRetryQueue(): Promise<number> {
    const workflowsToRetry = await this.getWorkflowsNeedingRetry();
    
    for (const workflow of workflowsToRetry) {
      try {
        this.logger.log(`Retrying workflow: ${workflow.id}`);
        await this.retryWorkflow(workflow.id);
      } catch (error) {
        this.logger.error(`Failed to retry workflow ${workflow.id}:`, error);
      }
    }

    return workflowsToRetry.length;
  }
}
