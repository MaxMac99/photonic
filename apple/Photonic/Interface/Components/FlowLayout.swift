//
//  FlowLayout.swift
//  Photonic
//
//  Created by Max Vissing on 16.03.25.
//

import SwiftUI

struct FlowLayout: Layout {
    func sizeThatFits(
        proposal: ProposedViewSize, subviews: Subviews, cache: inout ()
    ) -> CGSize {
        let sizes = subviews.map { $0.sizeThatFits(.unspecified) }

        var totalHeight: CGFloat = 0
        var totalWidth: CGFloat = 0
        var lineWidth: CGFloat = 0
        var lineHeight: CGFloat = 0

        for size in sizes {
            if lineWidth + size.width > proposal.width ?? 0 {
                totalHeight += lineHeight
                lineWidth = size.width
                lineHeight = size.height
            } else {
                lineWidth += size.width
                lineHeight = max(lineHeight, size.height)
            }
            totalWidth = max(totalWidth, lineWidth)
        }
        totalHeight += lineHeight

        return .init(width: totalWidth, height: totalHeight)
    }

    func placeSubviews(
        in bounds: CGRect,
        proposal: ProposedViewSize,
        subviews: Subviews,
        cache: inout ()
    ) {
        let sizes = subviews.map { $0.sizeThatFits(.unspecified) }

        var lineX = bounds.minX
        var lineY = bounds.minY
        var lineHeight: CGFloat = 0

        for (index, subview) in subviews.enumerated() {
            let size = sizes[index]
            if lineX + size.width > proposal.width ?? 0 {
                lineY += lineHeight
                lineHeight = 0
                lineX = bounds.minX
            }

            subview.place(
                at: .init(
                    x: lineX + size.width / 2,
                    y: lineY + size.height / 2
                ),
                anchor: .center,
                proposal: ProposedViewSize(size)
            )

            lineHeight = max(lineHeight, size.height)
            lineX += size.width
        }
    }
}

#Preview {
    FlowLayout {
        ForEach(
            ["Test1", "Recents", "Today", "Me", "Something really weird"],
            id: \.self
        ) { tag in
            HStack {
                Text(tag)
                Image(systemName: "xmark")
            }
            .padding(.vertical, 8)
            .padding(.horizontal)
            .background(
                Capsule()
                    .fill(.green.opacity(0.3))
                    .stroke(.green, lineWidth: 1)
            )
            .padding(4)
        }
    }
}
