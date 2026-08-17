namespace hfst
{
  hfst_ol::LocationVectorVector pmatch_locate
  (hfst_ol::PmatchContainer * cont,
   const std::string & input,
   double time_cutoff = 0.0)
  {
    return cont->locate(input, time_cutoff);
  }

  hfst_ol::LocationVectorVector pmatch_locate
  (hfst_ol::PmatchContainer * cont,
   const std::string & input,
   double time_cutoff,
   float weight_cutoff)
  {
    return cont->locate(input, time_cutoff, weight_cutoff);
  }

  std::ostringstream pmatch_tokenize_ostringstream;

  std::string pmatch_get_tokenized_output
  (hfst_ol::PmatchContainer * cont,
   const std::string & input_text,
   const std::string & output_format,
   int * max_weight_classes,
   bool dedupe,
   bool print_weights,
   bool print_all,
   double time_cutoff,
   bool verbose,
   float beam,
   bool tokenize_multichar)
  {
    pmatch_tokenize_ostringstream.str("");
    hfst_ol_tokenize::TokenizeSettings settings;
    if (output_format == "tokenize")
      settings.output_format=hfst_ol_tokenize::OutputFormat::tokenize;
    else if (output_format == "space_separated")
      settings.output_format=hfst_ol_tokenize::OutputFormat::space_separated;
    else if (output_format == "xerox")
      settings.output_format=hfst_ol_tokenize::OutputFormat::xerox;
    else if (output_format == "cg")
      settings.output_format=hfst_ol_tokenize::OutputFormat::cg;
    else if (output_format == "finnpos")
      settings.output_format=hfst_ol_tokenize::OutputFormat::finnpos;
    else if (output_format == "giellacg")
      settings.output_format=hfst_ol_tokenize::OutputFormat::giellacg;
    else if (output_format == "conllu")
      settings.output_format=hfst_ol_tokenize::OutputFormat::conllu;
    else
      throw "output_format not recognized";
    if (max_weight_classes == NULL)
      settings.max_weight_classes = std::numeric_limits<int>::max();
    else
      settings.max_weight_classes = *max_weight_classes;
    settings.dedupe = dedupe;
    settings.print_weights = print_weights;
    settings.print_all = print_all;
    settings.time_cutoff = time_cutoff;
    settings.verbose = verbose;
    settings.beam = beam;
    settings.tokenize_multichar = tokenize_multichar;
    hfst_ol_tokenize::match_and_print(*cont, pmatch_tokenize_ostringstream, input_text, settings);
    return pmatch_tokenize_ostringstream.str();
  }
}
