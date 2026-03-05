import {
  Entity,
  Column,
  PrimaryGeneratedColumn,
  CreateDateColumn,
  UpdateDateColumn,
} from 'typeorm';

@Entity('consents')
export class Consent {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', nullable: false })
  userId: string;

  @Column({ type: 'varchar', nullable: false })
  purpose: string;

  @Column({ type: 'boolean', default: false })
  granted: boolean;

  @Column({ type: 'timestamp', nullable: true })
  revokedAt?: Date;

  @Column({ type: 'varchar', nullable: true })
  ipAddress?: string;

  @Column({ type: 'varchar', nullable: true })
  userAgent?: string;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;
}
