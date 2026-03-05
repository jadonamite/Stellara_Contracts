import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { Tenant } from './entities/tenant.entity';
import { TenantConfig } from './entities/tenant-config.entity';
import { TenantUsage } from './entities/tenant-usage.entity';
import { TenantInvitation } from './entities/tenant-invitation.entity';

@Module({
  imports: [
    TypeOrmModule.forFeature([
      Tenant,
      TenantConfig,
      TenantUsage,
      TenantInvitation,
    ]),
  ],
  providers: [],
  exports: [TypeOrmModule],
})
export class TenantModule {}
