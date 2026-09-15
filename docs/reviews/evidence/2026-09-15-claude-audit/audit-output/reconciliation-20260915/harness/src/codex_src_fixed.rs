//! Isolated experimental replacement of Rubato's whole-buffer convenience loop.
//! Uses the same FFT resampler, with explicit delay removal after all output.
use audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft,FixedSync,Resampler,Indexing};
pub fn convert(x:&[f32],from:u32,to:u32,ch:usize)->Vec<f32>{
 if from==to{return x.to_vec();}if x.is_empty(){return vec![];}
 let frames=x.len()/ch;let input=InterleavedSlice::new(x,ch,frames).unwrap();
 let mut rs=Fft::<f32>::new(from as usize,to as usize,2048,1,ch,FixedSync::Both).unwrap();
 let expected=((frames as u128 * to as u128).div_ceil(from as u128)) as usize;
 let delay=rs.output_delay();let cap=expected+delay+rs.output_frames_max();let mut y=vec![0.0;cap*ch];
 {let mut output=InterleavedSlice::new_mut(&mut y,ch,cap).unwrap();let mut used=0;let mut written=0;
 while written<expected+delay{
  let wanted=rs.input_frames_next();let take=(frames-used).min(wanted);
  let index=Indexing{input_offset:used,output_offset:written,active_channels_mask:None,partial_len:if take<wanted{Some(take)}else{None}};
  let (_,n)=rs.process_into_buffer(&input,&mut output,Some(&index)).unwrap();used+=take;written+=n;
 }}
 y.copy_within(delay*ch..(delay+expected)*ch,0);y.truncate(expected*ch);y
}
