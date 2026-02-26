import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
} from 'typeorm';

@Entity('login_nonces')
export class LoginNonce {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ unique: true })
  nonce: string;

  @Column()
  publicKey: string;

  @Column()
  expiresAt: Date;

  @Column({ default: false })
  used: boolean;

  @CreateDateColumn()
  createdAt: Date;
}
