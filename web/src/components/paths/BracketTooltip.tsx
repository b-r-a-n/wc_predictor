import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';

interface BracketTooltipProps {
  content: React.ReactNode;
  children: React.ReactElement;
  disabled?: boolean;
}

export function BracketTooltip({ content, children, disabled }: BracketTooltipProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const triggerRef = useRef<HTMLDivElement>(null);
  const tooltipRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isVisible || !triggerRef.current || !tooltipRef.current) return;

    const updatePosition = () => {
      const triggerRect = triggerRef.current!.getBoundingClientRect();
      const tooltipRect = tooltipRef.current!.getBoundingClientRect();

      // Position above the trigger, centered
      let x = triggerRect.left + triggerRect.width / 2 - tooltipRect.width / 2;
      let y = triggerRect.top - tooltipRect.height - 8;

      // Keep within viewport bounds
      const padding = 8;
      if (x < padding) x = padding;
      if (x + tooltipRect.width > window.innerWidth - padding) {
        x = window.innerWidth - tooltipRect.width - padding;
      }
      if (y < padding) {
        // If no room above, show below
        y = triggerRect.bottom + 8;
      }

      setPosition({ x, y });
    };

    updatePosition();
    window.addEventListener('scroll', updatePosition, true);
    window.addEventListener('resize', updatePosition);

    return () => {
      window.removeEventListener('scroll', updatePosition, true);
      window.removeEventListener('resize', updatePosition);
    };
  }, [isVisible]);

  if (disabled) {
    return children;
  }

  const handleMouseEnter = () => setIsVisible(true);
  const handleMouseLeave = () => setIsVisible(false);

  return (
    <>
      <div
        ref={triggerRef}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        className="inline-block"
      >
        {children}
      </div>
      {isVisible &&
        createPortal(
          <div
            ref={tooltipRef}
            className="fixed z-50 px-3 py-2 text-sm bg-gray-900 text-white rounded-lg shadow-lg pointer-events-none"
            style={{
              left: position.x,
              top: position.y,
            }}
          >
            {content}
            <div
              className="absolute w-2 h-2 bg-gray-900 rotate-45"
              style={{
                left: '50%',
                bottom: '-4px',
                marginLeft: '-4px',
              }}
            />
          </div>,
          document.body
        )}
    </>
  );
}
