import {
  IsString,
  IsUrl,
  IsOptional,
  IsInt,
  Min,
  Max,
  IsBoolean,
  Length,
  IsArray,
  IsEnum,
} from 'class-validator';
import { Transform } from 'class-transformer';
import { EventType } from '../types/stellar.types';

export class UpdateConsumerDto {
  @IsOptional()
  @IsString()
  @Length(1, 100)
  name?: string;

  @IsOptional()
  @IsUrl({ require_protocol: true })
  @Length(1, 500)
  url?: string;

  @IsOptional()
  @IsString()
  @Length(1, 100)
  secret?: string;

  @IsOptional()
  @IsInt()
  @Min(0)
  @Max(20)
  @Transform(({ value }) => parseInt(value, 10))
  maxRetries?: number;

  @IsOptional()
  @IsInt()
  @Min(1000)
  @Max(30000)
  @Transform(({ value }) => parseInt(value, 10))
  timeoutMs?: number;

  @IsOptional()
  @IsBoolean()
  isActive?: boolean;

  @IsOptional()
  @IsArray()
  @IsEnum(EventType, { each: true })
  eventTypes?: EventType[];

  @IsOptional()
  @IsArray()
  @IsString({ each: true })
  contractIds?: string[];

  @IsOptional()
  @IsArray()
  @IsString({ each: true })
  accounts?: string[];
}
