import {
  Entity,
  Column,
  PrimaryGeneratedColumn,
  CreateDateColumn,
  UpdateDateColumn,
} from 'typeorm';

@Entity('analytics_metrics')
export class AnalyticsMetric {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', nullable: false })
  metricName: string;

  @Column({ type: 'json', nullable: true })
  metricValue: any;

  @Column({ type: 'varchar', nullable: true })
  userId?: string;

  @Column({ type: 'varchar', nullable: true })
  tenantId?: string;

  @Column({ type: 'timestamp', nullable: false })
  timestamp: Date;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;
}
