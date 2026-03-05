import {
  Entity,
  Column,
  PrimaryGeneratedColumn,
  CreateDateColumn,
  UpdateDateColumn,
} from 'typeorm';

@Entity('tenant_usage')
export class TenantUsage {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', nullable: false })
  tenantId: string;

  @Column({ type: 'varchar', nullable: false })
  metric: string;

  @Column({ type: 'bigint', nullable: false })
  value: number;

  @Column({ type: 'varchar', nullable: true })
  unit?: string;

  @Column({ type: 'date', nullable: false })
  periodStart: Date;

  @Column({ type: 'date', nullable: false })
  periodEnd: Date;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;
}
