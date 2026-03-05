import {
  Entity,
  Column,
  PrimaryGeneratedColumn,
  ManyToOne,
  JoinColumn,
  CreateDateColumn,
  UpdateDateColumn,
} from 'typeorm';
import { Tenant } from './tenant.entity';

@Entity('tenant_configs')
export class TenantConfig {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', nullable: false })
  tenantId: string;

  @Column({ type: 'varchar', nullable: false })
  configKey: string;

  @Column({ type: 'text', nullable: true })
  configValue?: string;

  @Column({ type: 'varchar', nullable: true })
  valueType: 'string' | 'number' | 'boolean' | 'json' = 'string';

  @Column({ type: 'boolean', default: true })
  isActive: boolean;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;
}
