import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { Consent } from './entities/consent.entity';

@Module({
  imports: [TypeOrmModule.forFeature([Consent])],
  providers: [],
  exports: [TypeOrmModule],
})
export class GdprModule {}
