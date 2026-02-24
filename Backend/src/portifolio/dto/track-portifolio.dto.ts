import {
  IsString,
  IsNotEmpty,
  IsOptional,
  IsEnum,
  IsArray,
  IsUrl,
} from 'class-validator';
import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';
import { PostStatus, PostType } from '../entities/post.entity';

export class CreatePostDto {
  @ApiProperty({
    description: 'Post title',
    example: 'Top 10 Betting Strategies for Beginners',
  })
  @IsString()
  @IsNotEmpty()
  title: string;

  @ApiProperty({
    description: 'Post content (supports markdown)',
    example: 'In this comprehensive guide, we will explore...',
  })
  @IsString()
  @IsNotEmpty()
  content: string;

  @ApiPropertyOptional({
    description: 'Short excerpt or summary of the post',
    example:
      'Learn the essential betting strategies every beginner should know',
  })
  @IsString()
  @IsOptional()
  excerpt?: string;

  @ApiPropertyOptional({
    description: 'URL-friendly slug for the post',
    example: 'top-10-betting-strategies-beginners',
  })
  @IsString()
  @IsOptional()
  slug?: string;

  @ApiPropertyOptional({
    description: 'URL to the featured/cover image',
    example: 'https://example.com/images/betting-strategies.jpg',
  })
  @IsUrl()
  @IsOptional()
  featuredImage?: string;

  @ApiPropertyOptional({
    description: 'Publication status of the post',
    enum: PostStatus,
    example: PostStatus.PUBLISHED,
    default: PostStatus.DRAFT,
  })
  @IsEnum(PostStatus)
  @IsOptional()
  status?: PostStatus;

  @ApiPropertyOptional({
    description: 'Type/category of the post',
    enum: PostType,
    example: PostType.ARTICLE,
    default: PostType.ARTICLE,
  })
  @IsEnum(PostType)
  @IsOptional()
  type?: PostType;

  @ApiPropertyOptional({
    description: 'Tags for categorization and search',
    example: ['betting', 'strategies', 'beginners', 'tips'],
    type: [String],
  })
  @IsArray()
  @IsString({ each: true })
  @IsOptional()
  tags?: string[];

  @ApiPropertyOptional({
    description: 'Additional metadata for the post',
    example: { readingTime: '5 min', difficulty: 'beginner' },
  })
  @IsOptional()
  metadata?: Record<string, any>;
}
