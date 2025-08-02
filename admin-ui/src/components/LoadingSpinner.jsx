import React from 'react';
import { Loader2 } from 'lucide-react';

export const LoadingSpinner = ({ 
  size = 'default', 
  text = 'Loading...', 
  className = '',
  overlay = false 
}) => {
  const sizeClasses = {
    sm: 'w-4 h-4',
    default: 'w-8 h-8',
    lg: 'w-12 h-12',
    xl: 'w-16 h-16'
  };

  const spinner = (
    <div className={`flex flex-col items-center justify-center space-y-2 ${className}`}>
      <Loader2 className={`${sizeClasses[size]} animate-spin text-gray-600`} />
      {text && <p className="text-sm text-gray-500">{text}</p>}
    </div>
  );

  if (overlay) {
    return (
      <div className="absolute inset-0 bg-white bg-opacity-75 flex items-center justify-center z-10">
        {spinner}
      </div>
    );
  }

  return spinner;
};

export const TableLoadingSpinner = ({ rows = 5, columns = 4 }) => (
  <div className="space-y-3">
    {Array.from({ length: rows }).map((_, i) => (
      <div key={i} className="flex space-x-4">
        {Array.from({ length: columns }).map((_, j) => (
          <div
            key={j}
            className="h-4 bg-gray-200 rounded animate-pulse flex-1"
          />
        ))}
      </div>
    ))}
  </div>
);

export const CardLoadingSpinner = () => (
  <div className="space-y-3">
    <div className="h-6 bg-gray-200 rounded animate-pulse w-1/3" />
    <div className="h-4 bg-gray-200 rounded animate-pulse w-full" />
    <div className="h-4 bg-gray-200 rounded animate-pulse w-2/3" />
  </div>
);

export default LoadingSpinner;