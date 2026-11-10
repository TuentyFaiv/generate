import SwiftUI

/// The smallest UI building block: a label-sized piece of content.
struct ExampleAtom<Content: View>: View {
    /// Visual emphasis.
    enum Tone {
        case neutral, accent
    }

    var tone: Tone = .neutral
    @ViewBuilder var content: Content

    var body: some View {
        content
            .font(.caption.weight(.semibold))
            .padding(.horizontal, 8)
            .padding(.vertical, 2)
            .foregroundStyle(tone == .accent ? AnyShapeStyle(.white) : AnyShapeStyle(.secondary))
            .background(tone == .accent ? AnyShapeStyle(.tint) : AnyShapeStyle(.quaternary), in: .capsule)
    }
}

#Preview {
    HStack {
        ExampleAtom { Text("Draft") }
        ExampleAtom(tone: .accent) { Text("New") }
    }
    .padding()
}
